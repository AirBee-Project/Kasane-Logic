#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::TemporalId;

impl TemporalId {
    /// 2つのTemporalIdの重なる時間範囲（Intersection）を計算して返す。
    ///
    /// 2つの時間区間の交差が存在し、かつ [`TemporalId`] の定義する時間間隔で
    /// 正確に表現できる場合、交差を表す新しい [`TemporalId`] を返す。
    /// 重なりがない場合や、交差が時間間隔の境界に合致しない場合は `None` を返す。
    ///
    /// # パラメーター
    ///
    /// * `other` — 交差を計算する相手の [`TemporalId`]。
    ///
    /// # 戻り値
    ///
    /// 交差を表す [`TemporalId`] が存在する場合は `Some(id)`、
    /// そうでない場合は `None` を返す。
    ///
    /// # 例
    ///
    /// 交差がある場合:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 5).unwrap();  // [18000, 21599]
    /// let id2 = TemporalId::new(3600, 6).unwrap();  // [21600, 24999]
    /// assert_eq!(id1.intersection(&id2), None);     // No overlap
    ///
    /// let id3 = TemporalId::new(1, 18000).unwrap(); // [18000, 18000]
    /// let inter = id1.intersection(&id3);
    /// assert!(inter.is_some());
    /// # }
    /// ```
    pub fn intersection(&self, other: &TemporalId) -> Option<TemporalId> {
        let self_start = self.start_unixstamp();
        let self_end_excl = self.end_unixtime_exclusive() as u64;
        let other_start = other.start_unixstamp();
        let other_end_excl = other.end_unixtime_exclusive() as u64;

        let inter_start = self_start.max(other_start);
        let inter_end_excl = self_end_excl.min(other_end_excl);

        if inter_start >= inter_end_excl {
            return None;
        }

        // Try to find a TemporalId that exactly represents this intersection
        for &interval in &Self::TEMPORAL_I {
            if inter_start.is_multiple_of(interval)
                && (inter_end_excl - inter_start).is_multiple_of(interval)
            {
                let t = inter_start / interval;
                if interval * (t + 1) == inter_end_excl {
                    return TemporalId::new(interval, t).ok();
                }
            }
        }

        None
    }

    /// 相手の [`TemporalId`] との差集合（self - other）を計算し、
    /// イテレータとして返す。
    ///
    /// `self` の時間範囲から `other` の時間範囲を除いた部分を計算する。
    /// 結果は0個、1個、または2個の [`TemporalId`] となる。
    ///
    /// 差集合の各要素は、元々の `self` と同じ時間間隔を持つ場合、
    /// その間隔で表現される。異なる間隔を持つ場合は、
    /// より小さい間隔への分割が行われる可能性がある。
    ///
    /// # パラメーター
    ///
    /// * `other` — `self` から除外する [`TemporalId`]。
    ///
    /// # 戻り値
    ///
    /// 差集合を表す [`TemporalId`] のイテレータ。
    ///
    /// # 例
    ///
    /// 重なりがない場合（self全体が返される）:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 0).unwrap();   // [0, 3599]
    /// let id2 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 1);
    /// assert_eq!(diff[0], id1);
    /// # }
    /// ```
    ///
    /// 左右に分かれる場合:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let id2 = TemporalId::new(1, 19800).unwrap();  // [19800, 19800]（中間の1秒）
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// // 差集合は複数のピースに分かれる可能性がある
    /// # }
    /// ```
    ///
    /// 完全に包含される場合（空のイテレータ）:
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::TemporalId;
    /// let id1 = TemporalId::new(1, 19800).unwrap();  // [19800, 19800]
    /// let id2 = TemporalId::new(3600, 5).unwrap();   // [18000, 21599]
    /// let diff: Vec<_> = id1.difference(&id2).collect();
    /// assert_eq!(diff.len(), 0);
    /// # }
    /// ```
    pub fn difference(&self, other: &TemporalId) -> impl Iterator<Item = TemporalId> {
        let mut result = Vec::new();

        let self_start = self.start_unixstamp();
        let self_end_excl = self.end_unixtime_exclusive() as u64;
        let other_start = other.start_unixstamp();
        let other_end_excl = other.end_unixtime_exclusive() as u64;

        if other_end_excl <= self_start || other_start >= self_end_excl {
            result.push(self.clone());
            return result.into_iter();
        }

        if self_start < other_start {
            let left_duration = other_start - self_start;
            let mut found = false;

            for &interval in &Self::TEMPORAL_I {
                if self_start.is_multiple_of(interval) && left_duration.is_multiple_of(interval) {
                    let t = self_start / interval;
                    if interval * (t + 1) == other_start
                        && let Ok(id) = TemporalId::new(interval, t)
                    {
                        result.push(id);
                        found = true;
                        break;
                    }
                }
            }

            // If left part cannot be expressed, split by seconds
            if !found {
                for ts in self_start..other_start {
                    if let Ok(id) = TemporalId::new(1, ts) {
                        result.push(id);
                    }
                }
            }
        }

        // Right part (other_end_excl to self_end_excl)
        if self_end_excl > other_end_excl {
            let right_duration = self_end_excl - other_end_excl;
            let mut found = false;

            for &interval in &Self::TEMPORAL_I {
                if other_end_excl.is_multiple_of(interval)
                    && right_duration.is_multiple_of(interval)
                {
                    let t = other_end_excl / interval;
                    if interval * (t + 1) == self_end_excl
                        && let Ok(id) = TemporalId::new(interval, t)
                    {
                        result.push(id);
                        found = true;
                        break;
                    }
                }
            }

            // If right part cannot be expressed, split by seconds
            if !found {
                for ts in other_end_excl..self_end_excl {
                    if let Ok(id) = TemporalId::new(1, ts) {
                        result.push(id);
                    }
                }
            }
        }

        result.into_iter()
    }
}
