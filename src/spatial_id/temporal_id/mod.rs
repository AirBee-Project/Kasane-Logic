use crate::Error;
pub mod impls;
pub mod segment;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct TemporalId {
    ///時間間隔(秒)
    i: u64,
    ///時間インデックス [開始, 終了]
    t: [u64; 2],
}

impl TemporalId {
    /// 新しい[TemporalId]を作成する
    pub fn new(i: u64, mut t: [u64; 2]) -> Result<Self, Error> {
        // 間隔が0の場合はエラー（ゼロ除算等の原因になるため）
        if i == 0 {
            // Error::InvalidInterval のようなバリアントを追加することをお勧めします
            return Err(Error::TOutOfRange { i, t: t[1] });
        }

        if t[0] > t[1] {
            t.swap(0, 1);
        }

        // この区間に含まれる「最後の1秒」の実時間を計算 (Inclusive End)
        // 例: i=10, t[1]=2 の場合、最後の秒は (10 * 2) + 10 - 1 = 29秒
        let inclusive_end = (i as u128) * (t[1] as u128) + (i as u128) - 1;

        // 最後の秒が u64::MAX を超えていればエラー
        if inclusive_end > u64::MAX as u128 {
            return Err(Error::TOutOfRange { i, t: t[1] });
        }

        Ok(Self { i, t })
    }

    /// Unix時間の全範囲 (0 から u64::MAX 秒まで) を表す[TemporalId]を返す
    pub fn whole() -> Self {
        Self {
            i: 1,
            t: [0, u64::MAX],
        }
    }

    /// 全範囲を表しているか判定する
    pub fn is_whole(&self) -> bool {
        self.i == 1 && self.t == [0, u64::MAX]
    }

    /// 実際の開始時刻 (Unix時間の経過秒数) を返す
    pub fn start_unixstamp(&self) -> u64 {
        self.i * self.t[0]
    }

    /// 区間が包含する「最後の1秒」の時刻 (Inclusive) を返す。
    /// 仕様により、この値は必ず 0 から u64::MAX の間に収まります。
    pub fn end_unixstamp_inclusive(&self) -> u64 {
        // new() でチェック済みのためダウンキャストは安全
        ((self.i as u128) * (self.t[1] as u128) + (self.i as u128) - 1) as u64
    }

    /// 次の区間の開始時刻、すなわち「終了時刻の直後」 (Exclusive) を返す。
    /// u64::MAXまでカバーしている場合、戻り値は (u64::MAX + 1) となるため u128 で返します。
    pub fn end_unixtime_exclusive(&self) -> u128 {
        (self.i as u128) * ((self.t[1] as u128) + 1)
    }

    /// 秒単位で長さを返す
    pub fn length_seconds(&self) -> u128 {
        self.end_unixtime_exclusive() - (self.start_unixstamp() as u128)
    }

    /// 情報を失わないまま `i`（時間間隔）を最大化します。
    pub fn optimize_i(&mut self) {
        let s = self.start_unixstamp() as u128;
        let e = self.end_unixtime_exclusive(); // 終端は Exclusive (最大 2^64) で計算

        let mut new_i = Self::gcd(s, e);
        if new_i == 0 {
            return;
        }

        // 万が一、最大公約数が u64::MAX を超える場合 (例: whole() の 2^64)
        // u64に収まるまで 2 で割り続ける (Z=64の設計上、2で割れるはずです)
        while new_i > u64::MAX as u128 {
            if new_i % 2 == 0 {
                new_i /= 2;
            } else {
                return; // u64に収められない場合は最適化を諦める（安全へのフォールバック）
            }
        }

        let new_i_u64 = new_i as u64;

        *self = Self {
            i: new_i_u64,
            t: [(s / new_i) as u64, (e / new_i) as u64 - 1],
        };
    }

    /// 最大公約数を求める内部関数 (u128対応)
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            a %= b;
            std::mem::swap(&mut a, &mut b);
        }
        a
    }
}
