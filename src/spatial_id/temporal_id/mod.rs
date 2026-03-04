use std::num::{NonZero, NonZeroU64};

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
    /// 指定された値から [`SingleId`] を構築します。
    ///
    /// # パラメーター
    ///
    /// # バリテーション
    /// i×tの値が0-u64::MAXに収まらずオーバーフローする場合は、[Error::TOutOfRange]を返す。
    pub fn new(i: NonZeroU64, mut t: [u64; 2]) -> Result<Self, Error> {
        // 間隔が0の場合はエラー（ゼロ除算等の原因になるため）

        if t[0] > t[1] {
            t.swap(0, 1);
        }

        let inclusive_end = (i.get() as u128) * (t[1] as u128) + (i.get() as u128) - 1;

        if inclusive_end > u64::MAX as u128 {
            return Err(Error::TOutOfRange { i, t: t[1] });
        }

        Ok(Self { i, t })
    }

    /// Unix時間の全範囲 (0 から u64::MAX 秒まで) を表す[TemporalId]を返す。
    pub fn whole() -> Self {
        Self {
            i: NonZero::new(1).unwrap(),
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
