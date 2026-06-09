#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::SingleId;

impl SingleId {
    /// 相手の [`SingleId`] との差集合（`self - other`）を計算し、イテレータとして返す。
    ///
    /// 空間と時間の両方を考慮し、`other` に含まれる領域を取り除いた残りを、必要に応じて細かい [`SingleId`] に分割して返す。
    /// 重なりがない場合は `self` をそのまま 1 件返し、完全に含まれる場合は空になる。
    ///
    /// # パラメーター
    /// * `other` - 差し引く相手の [`SingleId`] である。
    ///
    /// # 動作コスト
    /// 空間的な分割回数は、`self` と `other` の重なりを解消するために必要なズーム差に比例する。
    /// 時間的な差分は [`TemporalId::difference`] の結果個数に比例する。
    ///
    /// # 動作例
    ///
    /// 重なりがない場合:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let left = SingleId::new(2, 1, 1, 1).unwrap();
    /// let right = SingleId::new(2, 2, 1, 1).unwrap();
    /// let diff: Vec<_> = left.difference(&right).collect();
    /// assert_eq!(diff, vec![left]);
    /// ```
    ///
    /// 一方が他方を含む場合:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let parent = SingleId::new(1, 0, 0, 0).unwrap();
    /// let child = SingleId::new(2, 0, 1, 1).unwrap();
    /// let diff: Vec<_> = parent.difference(&child).collect();
    /// assert_eq!(diff.len(), 7);
    /// ```
    pub fn difference(&self, other: &Self) -> impl Iterator<Item = Self> {
        let mut results = Vec::new();

        let intersect = match self.intersection(other) {
            Some(i) => i,
            None => {
                results.push(self.clone());
                return results.into_iter();
            }
        };

        if self == &intersect {
            return results.into_iter();
        }

        let mut current = self.clone();

        while current.z() < intersect.z() {
            let next_z = current.z() + 1;
            let children: Vec<_> = current.spatial_children_at_zoom(next_z).unwrap().collect();

            for child in children {
                if child.intersection(&intersect).is_some() {
                    current = child;
                } else {
                    results.push(child);
                }
            }
        }

        for t_diff in current.temporal_id.difference(&other.temporal_id) {
            let mut diff_id = current.clone();
            diff_id.temporal_id = t_diff;
            results.push(diff_id);
        }

        results.into_iter()
    }

    /// 2つの [`SingleId`] の重なっている領域（Intersection）を計算して返す。
    ///
    /// 空間軸については、より深いズームレベル側の座標を浅い側に合わせて比較し、両者が同じ領域に属する場合に重なりありと判定する。
    /// 時間軸については [`TemporalId::intersection`] を用いて重なりを求める。
    /// 重なりがない場合は `None` を返す。
    ///
    /// # パラメーター
    /// * `other` - 交差判定する相手の [`SingleId`] である。
    ///
    /// # 動作コスト
    /// 空間軸の判定は 3 次元それぞれについて一定回数のビットシフト比較で完了する。
    /// 時間軸の判定は [`TemporalId::intersection`] の計算量に従う。
    ///
    /// # 動作例
    ///
    /// 祖先と子孫の重なり:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let ancestor = SingleId::new(2, 1, 1, 1).unwrap();
    /// let descendant = SingleId::new(3, 2, 2, 3).unwrap();
    /// assert_eq!(ancestor.intersection(&descendant).unwrap(), descendant);
    /// ```
    ///
    /// 重なりがない場合:
    /// ```
    /// # use kasane_logic::SingleId;
    /// let left = SingleId::new(3, 1, 1, 1).unwrap();
    /// let right = SingleId::new(3, 4, 1, 1).unwrap();
    /// assert!(left.intersection(&right).is_none());
    /// ```
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        let (deep, shallow) = if self.z() > other.z() {
            (self, other)
        } else {
            (other, self)
        };

        let shift = deep.z() - shallow.z();

        if (deep.f() >> shift) == shallow.f()
            && (deep.x() >> shift) == shallow.x()
            && (deep.y() >> shift) == shallow.y()
        {
            let temporal_id = self.temporal_id.intersection(&other.temporal_id)?;

            let mut result = deep.clone();
            result.temporal_id = temporal_id;
            Some(result)
        } else {
            None
        }
    }
}
