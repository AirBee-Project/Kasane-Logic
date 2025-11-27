/// 時間区間を効率的に管理するモジュール
///
/// BitVecによる階層構造ではなく、直接的な区間表現を使用することで
/// 時間データの管理を効率化する

mod relation;

pub use relation::TimeIntervalRelation;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::bit_vec::BitVec;

/// 時間区間を表す構造体
///
/// 開始時刻と終了時刻を直接保持し、区間演算を効率的に行う
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeInterval {
    /// 開始時刻（含む）
    pub start: u64,
    /// 終了時刻（含む）
    pub end: u64,
}

impl TimeInterval {
    /// 新しい時間区間を作成
    ///
    /// start <= end が保証される（逆の場合は自動的に入れ替え）
    pub fn new(start: u64, end: u64) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// 全時間範囲を表す区間を作成
    pub fn all() -> Self {
        Self {
            start: 0,
            end: u64::MAX - 1,
        }
    }

    /// 単一時点を表す区間を作成
    pub fn point(t: u64) -> Self {
        Self { start: t, end: t }
    }

    /// 区間の長さを返す
    pub fn len(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }

    /// 区間が空かどうかを返す（常にfalse、最小でも長さ1）
    pub fn is_empty(&self) -> bool {
        false
    }

    /// 指定した時刻が区間内に含まれるかどうかを返す
    pub fn contains(&self, t: u64) -> bool {
        self.start <= t && t <= self.end
    }

    /// 二つの区間が重なるかどうかを返す
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }

    /// 二つの区間の共通部分を返す
    /// 重ならない場合はNoneを返す
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if self.overlaps(other) {
            Some(Self {
                start: self.start.max(other.start),
                end: self.end.min(other.end),
            })
        } else {
            None
        }
    }

    /// 二つの区間の和集合を返す
    /// 連続していない場合は2つの区間を返す
    pub fn union(&self, other: &Self) -> Vec<Self> {
        // 隣接または重なる場合は1つの区間
        if self.start <= other.end.saturating_add(1) && other.start <= self.end.saturating_add(1) {
            vec![Self {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            }]
        } else {
            // 連続していない場合は2つの区間
            if self.start < other.start {
                vec![*self, *other]
            } else {
                vec![*other, *self]
            }
        }
    }

    /// selfからotherを引いた残りの区間を返す
    pub fn subtract(&self, other: &Self) -> Vec<Self> {
        if !self.overlaps(other) {
            // 重ならない場合はself全体
            return vec![*self];
        }

        let mut result = Vec::new();

        // 左側の残り（underflowを防ぐためother.start > 0をチェック）
        if self.start < other.start && other.start > 0 {
            result.push(Self {
                start: self.start,
                end: other.start - 1,
            });
        }

        // 右側の残り（overflowを防ぐためother.end < u64::MAXをチェック）
        if self.end > other.end && other.end < u64::MAX {
            result.push(Self {
                start: other.end + 1,
                end: self.end,
            });
        }

        result
    }

    /// TimeIntervalからBitVecに変換
    ///
    /// 時間区間を16バイトのBitVecとしてエンコードする
    /// 最初の8バイトがstart、次の8バイトがendを表す
    pub fn to_bitvec(&self) -> BitVec {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend_from_slice(&self.start.to_be_bytes());
        bytes.extend_from_slice(&self.end.to_be_bytes());
        BitVec::from_vec(bytes)
    }

    /// BitVecからTimeIntervalに変換
    ///
    /// 16バイトのBitVecを時間区間にデコードする
    pub fn from_bitvec(bitvec: &BitVec) -> Option<Self> {
        if bitvec.0.len() < 16 {
            return None;
        }

        let start = u64::from_be_bytes([
            bitvec.0[0],
            bitvec.0[1],
            bitvec.0[2],
            bitvec.0[3],
            bitvec.0[4],
            bitvec.0[5],
            bitvec.0[6],
            bitvec.0[7],
        ]);

        let end = u64::from_be_bytes([
            bitvec.0[8],
            bitvec.0[9],
            bitvec.0[10],
            bitvec.0[11],
            bitvec.0[12],
            bitvec.0[13],
            bitvec.0[14],
            bitvec.0[15],
        ]);

        Some(Self { start, end })
    }
}

impl Ord for TimeInterval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start.cmp(&other.start) {
            std::cmp::Ordering::Equal => self.end.cmp(&other.end),
            ord => ord,
        }
    }
}

impl PartialOrd for TimeInterval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let interval = TimeInterval::new(10, 20);
        assert_eq!(interval.start, 10);
        assert_eq!(interval.end, 20);

        // 逆順でも正しくソート
        let interval2 = TimeInterval::new(20, 10);
        assert_eq!(interval2.start, 10);
        assert_eq!(interval2.end, 20);
    }

    #[test]
    fn test_contains() {
        let interval = TimeInterval::new(10, 20);
        assert!(interval.contains(10));
        assert!(interval.contains(15));
        assert!(interval.contains(20));
        assert!(!interval.contains(9));
        assert!(!interval.contains(21));
    }

    #[test]
    fn test_overlaps() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(15, 25);
        let c = TimeInterval::new(21, 30);
        let d = TimeInterval::new(5, 10);

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
        assert!(a.overlaps(&d));
    }

    #[test]
    fn test_intersection() {
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(15, 25);

        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter.start, 15);
        assert_eq!(inter.end, 20);

        let c = TimeInterval::new(21, 30);
        assert!(a.intersection(&c).is_none());
    }

    #[test]
    fn test_subtract() {
        let a = TimeInterval::new(10, 30);
        let b = TimeInterval::new(15, 20);

        let result = a.subtract(&b);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], TimeInterval::new(10, 14));
        assert_eq!(result[1], TimeInterval::new(21, 30));

        // 左側だけ残る
        let c = TimeInterval::new(20, 40);
        let result2 = a.subtract(&c);
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0], TimeInterval::new(10, 19));
    }

    #[test]
    fn test_bitvec_roundtrip() {
        let interval = TimeInterval::new(12345, 67890);
        let bitvec = interval.to_bitvec();
        let decoded = TimeInterval::from_bitvec(&bitvec).unwrap();
        assert_eq!(interval, decoded);
    }

    #[test]
    fn test_bitvec_max_values() {
        let interval = TimeInterval::new(0, u64::MAX - 1);
        let bitvec = interval.to_bitvec();
        let decoded = TimeInterval::from_bitvec(&bitvec).unwrap();
        assert_eq!(interval, decoded);
    }

    #[test]
    fn test_union() {
        // 隣接する区間
        let a = TimeInterval::new(10, 20);
        let b = TimeInterval::new(21, 30);
        let result = a.union(&b);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TimeInterval::new(10, 30));

        // 重なる区間
        let c = TimeInterval::new(10, 20);
        let d = TimeInterval::new(15, 25);
        let result2 = c.union(&d);
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0], TimeInterval::new(10, 25));

        // 離れた区間
        let e = TimeInterval::new(10, 20);
        let f = TimeInterval::new(30, 40);
        let result3 = e.union(&f);
        assert_eq!(result3.len(), 2);
    }

    #[test]
    fn test_all_and_point() {
        let all = TimeInterval::all();
        assert_eq!(all.start, 0);
        assert_eq!(all.end, u64::MAX - 1);

        let point = TimeInterval::point(100);
        assert_eq!(point.start, 100);
        assert_eq!(point.end, 100);
        assert_eq!(point.len(), 1);
    }

    #[test]
    fn test_subtract_edge_cases() {
        // other.start が 0 の場合（underflow防止テスト）
        let a = TimeInterval::new(0, 10);
        let b = TimeInterval::new(0, 5);
        let result = a.subtract(&b);
        // 左側には何も残らない（other.start == 0 なので）
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], TimeInterval::new(6, 10));

        // other.end が u64::MAX の場合（overflow防止テスト）
        let c = TimeInterval::new(10, u64::MAX);
        let d = TimeInterval::new(15, u64::MAX);
        let result2 = c.subtract(&d);
        // 右側には何も残らない（other.end == u64::MAX なので）
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0], TimeInterval::new(10, 14));

        // 完全に包含される場合
        let e = TimeInterval::new(10, 20);
        let f = TimeInterval::new(5, 25);
        let result3 = e.subtract(&f);
        assert_eq!(result3.len(), 0);
    }
}
