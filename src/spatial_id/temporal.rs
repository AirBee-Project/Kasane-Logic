use std::fmt::Display;

use crate::Error;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
///時間軸の起点 [1970 年 1 月 1 日 0:00] から一定の時間間隔 [任意指定 (単位：秒)] で時間軸を分割し、その時間間隔ごとに一意の識別子を付与する。
pub struct TemporalId {
    ///時間間隔(秒)
    i: u64,
    ///時間インデックス
    t: [u64; 2],
}

impl TemporalId {
    pub fn new(i: u64, mut t: [u64; 2]) -> Result<Self, Error> {
        //順序を入れ替える
        if t[0] > t[1] {
            t.swap(0, 1);
        }

        let end = (i as u128) * ((t[1] as u128) + 1);

        //終点がu64の範囲内で収まっていることをチェック
        if end > u64::MAX as u128 {
            return Err(Error::TOutOfRange { i, t: t[1] });
        }
        Ok(Self { i, t })
    }

    pub fn whole() -> Self {
        Self {
            i: u64::MAX,

            t: [0, 0],
        }
    }

    pub fn is_whole(&self) -> bool {
        self.start_unixstamp() == 0 && self.end_unixtime() == u64::MAX
    }

    /// 時間間隔(秒)を返す
    pub fn interval(&self) -> u64 {
        self.i
    }

    /// 開始インデックスを返す
    pub fn start_index(&self) -> u64 {
        self.t[0]
    }

    /// 終了インデックスを返す
    pub fn end_index(&self) -> u64 {
        self.t[1]
    }

    /// 実際の開始時刻 (Unix Time等からの経過秒数) を返す
    pub fn start_unixstamp(&self) -> u64 {
        self.i * self.t[0]
    }

    /// 実際の終了時刻 (Unix Time等からの経過秒数) を返す
    pub fn end_unixtime(&self) -> u64 {
        self.i * (self.t[1] + 1)
    }

    /// 秒単位で長さを返す
    pub fn length_seconds(&self) -> u64 {
        self.end_unixtime() - self.start_unixstamp()
    }
}

impl Display for TemporalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", self.i);

        if self.t[0] == self.t[1] {
            write!(f, "{}", self.t[0]);
        } else {
            write!(f, "{}:{}", self.t[0], self.t[1]);
        }

        Ok(())
    }
}
