use alloc::vec::Vec;

use crate::TemporalId;

#[cfg(not(feature = "temporal_id"))]
pub mod disabled;

#[cfg(feature = "temporal_id")]
impl TemporalId {
    /// 2つのTemporalIdの重なる時間範囲（Intersection）を返す。
    /// # 例
    ///
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600_u64, 5).unwrap();  // [18000, 21600)
    /// let id2 = TemporalId::new(3600_u64, 6).unwrap();  // [21600, 25200)
    /// assert_eq!(id1.intersection(id2), None);     // 重なりなし
    ///
    /// let id3 = TemporalId::new(1_u64, 18000).unwrap(); // [18000, 18001)
    /// assert_eq!(id1.intersection(id3), Some(id3.clone()));
    /// ```
    pub fn intersection(&self, other: TemporalId) -> Option<TemporalId> {
        if self.contains(other) {
            Some(other)
        } else if other.contains(*self) {
            Some(*self)
        } else {
            None
        }
    }

    /// 相手の [`TemporalId`] との差集合（self - other）を計算し、イテレータとして返す。
    /// # 例
    ///
    /// 重なりがない場合（self全体が返される）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600_u64, 0).unwrap();   // [0, 3600)
    /// let id2 = TemporalId::new(3600_u64, 5).unwrap();   // [18000, 21600)
    /// let diff: Vec<_> = id1.difference(id2).collect();
    /// assert_eq!(diff.len(), 1);
    /// assert_eq!(diff[0], id1);
    /// ```
    ///
    /// 完全に包含される場合（空のイテレータ）:
    /// ```
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(1_u64, 19800).unwrap();  // [19800, 19801)
    /// let id2 = TemporalId::new(3600_u64, 5).unwrap();   // [18000, 21600)
    /// let diff: Vec<_> = id1.difference(id2).collect();
    /// assert_eq!(diff.len(), 0);
    /// ```
    pub fn difference(&self, other: TemporalId) -> impl Iterator<Item = TemporalId> {
        let s0 = self.start_unixtime();
        let s1 = self.end_unixtime_exclusive();
        let o0 = other.start_unixtime();
        let o1 = other.end_unixtime_exclusive();

        let mut result = Vec::new();

        if o1 <= s0 || o0 >= s1 {
            result.push(*self);
            return result.into_iter();
        }

        let left_end = o0.min(s1);
        if s0 < left_end {
            result.extend(Self::from_range(s0..left_end).unwrap());
        }

        let right_start = o1.max(s0);
        if right_start < s1 {
            result.extend(Self::from_range(right_start..s1).unwrap());
        }

        result.into_iter()
    }

    /// ある範囲に限定した差集合 `(self ∩ window) − other` を返す。
    /// 集合論的に `(self ∩ window) − other = (self − other) ∩ window` と一致する。
    pub fn difference_clipped(&self, other: &TemporalId, window: &TemporalId) -> Vec<TemporalId> {
        let w0 = window.start_unixtime();
        let w1 = window.end_unixtime_exclusive();
        let s0 = self.start_unixtime().max(w0);
        let s1 = self.end_unixtime_exclusive().min(w1);

        let mut result = Vec::new();
        if s0 >= s1 {
            return result;
        }

        let o0 = other.start_unixtime();
        let o1 = other.end_unixtime_exclusive();

        if o1 <= s0 || o0 >= s1 {
            return Self::from_range(s0..s1).unwrap().collect();
        }

        let left_end = o0.min(s1);
        if s0 < left_end {
            result.extend(Self::from_range(s0..left_end).unwrap());
        }

        let right_start = o1.max(s0);
        if right_start < s1 {
            result.extend(Self::from_range(right_start..s1).unwrap());
        }

        result
    }

    /// `other` の時間範囲が `self` に完全に含まれるかを判定する。
    pub fn contains(&self, other: TemporalId) -> bool {
        self.start_unixtime() <= other.start_unixtime()
            && other.end_unixtime_exclusive() <= self.end_unixtime_exclusive()
    }
}
