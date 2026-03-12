use std::num::NonZeroU64;

use crate::Error;
pub mod impls;
pub mod segment;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
///[TemporalId]は時間IDの区間表現を表す型。
///
/// 内部的には下記のような構造体で構成されている。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用であり、実際の空間的な「大小」を意味するものではない。

pub struct TemporalId {
    ///時間間隔(秒)
    i: NonZeroU64,
    ///時間インデックス [開始, 終了]
    t: [u64; 2],
}

impl TemporalId {
    /// 指定された値から [`TemporalId`] を構築します。
    ///
    ///　各次元の与えられた2つの値は自動的に昇順に並び替えられ、常に `[min, max]` の形で内部に保持されます。
    ///
    /// # パラメーター
    /// * `i` — インターバル（0–64::MAXの範囲が有効）
    /// * `t1` — 時間方向の開始のTインデックス
    /// * `t2` — 時間方向の終了のTインデックス
    ///
    /// # バリテーション
    ///- i×tの値が0-u64::MAXに収まらずオーバーフローする場合は、[Error::TOutOfRange]を返す。
    /// - iの値が0の場合は[Error::TIntervalZero]を返す。
    ///
    /// IDの作成（単体）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(60, [1,1]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "60/1");
    /// ```
    ///
    /// IDの作成（範囲）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(60, [0,60]).unwrap();
    /// let s = format!("{}", id);
    /// assert_eq!(s, "60/0:60");
    /// ```
    ///
    /// オーバーフローの検知:
    /// ```
    /// # use kasane_logic::Error;
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(3600, [0,6000000000000000000]);
    /// assert_eq!(id, Err(Error::TOutOfRange{i:3600,t:6000000000000000000}));
    /// ```
    ///
    /// i=0の検知:
    /// ```
    /// # use kasane_logic::Error;
    /// # use kasane_logic::TemporalId;
    /// let id = TemporalId::new(0, [0,60]);
    /// assert_eq!(id, Err(Error::TIntervalZero));
    /// ```
    pub fn new(i: u64, mut t: [u64; 2]) -> Result<Self, Error> {
        let i_non_zero = NonZeroU64::new(i).ok_or(Error::TIntervalZero)?;

        if t[0] > t[1] {
            t.swap(0, 1);
        }

        let i_u128 = i_non_zero.get() as u128;
        let inclusive_end = i_u128 * (t[1] as u128) + i_u128 - 1;

        if inclusive_end > u64::MAX as u128 {
            return Err(Error::TOutOfRange { i: i, t: t[1] });
        }

        Ok(Self { i: i_non_zero, t })
    }

    /// Unix時間の全範囲 (0 から u64::MAX 秒まで) を表す以下の[TemporalId]を返す。
    ///
    /// ```ignore
    /// # use std::num::NonZero;
    /// # use kasane_logic::TemporalId;
    /// let result = TemporalId {
    ///   i: NonZero::new(u64::MAX).unwrap(),
    ///   t: [0,0],
    /// };
    /// ```
    ///
    /// IDの作成:
    /// ```
    /// # use kasane_logic::TemporalId;
    ///
    /// let id=TemporalId::whole();
    /// assert_eq!(id.start_unixstamp(),0);
    /// assert_eq!(id.end_unixstamp_inclusive(),u64::MAX);
    /// ```
    pub fn whole() -> Self {
        Self {
            i: NonZeroU64::new(1).unwrap(),
            t: [0, u64::MAX],
        }
    }

    /// 全範囲を表しているか判定する
    pub fn is_whole(&self) -> bool {
        self.start_unixstamp() == 0 && self.end_unixtime_exclusive() == (u64::MAX as u128) + 1
    }

    /// 実際の開始時刻 (Unix時間の経過秒数) を返す
    pub fn start_unixstamp(&self) -> u64 {
        self.i.get() * self.t[0]
    }

    /// 区間が包含する「最後の1秒」の時刻 (Inclusive) を返す。
    /// 仕様により、この値は必ず 0 から u64::MAX の間に収まります。
    pub fn end_unixstamp_inclusive(&self) -> u64 {
        // new() でチェック済みのためダウンキャストは安全
        ((self.i.get() as u128) * (self.t[1] as u128) + (self.i.get() as u128) - 1) as u64
    }

    /// 次の区間の開始時刻、すなわち「終了時刻の直後」 (Exclusive) を返す。
    /// u64::MAXまでカバーしている場合、戻り値は (u64::MAX + 1) となるため u128 で返します。
    pub fn end_unixtime_exclusive(&self) -> u128 {
        (self.i.get() as u128) * ((self.t[1] as u128) + 1)
    }

    /// 秒単位で長さを返す
    pub fn length_seconds(&self) -> u128 {
        self.end_unixtime_exclusive() - (self.start_unixstamp() as u128)
    }

    /// 情報を失わないまま `i`（時間間隔）を最大化します。
    pub fn optimize_i(&mut self) {
        let s = self.start_unixstamp() as u128;
        let e = self.end_unixtime_exclusive();

        let mut new_i = Self::gcd(s, e);

        if new_i == 0 {
            return;
        }

        while new_i > u64::MAX as u128 {
            if new_i % 2 == 0 {
                new_i /= 2;
            } else {
                return;
            }
        }

        let new_i_u64 = new_i as u64;

        *self = Self {
            i: NonZeroU64::new(new_i_u64).expect("Optimized interval cannot be zero"),
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
