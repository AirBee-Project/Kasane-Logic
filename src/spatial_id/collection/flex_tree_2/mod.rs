use crate::FlexId;
use alloc::sync::Arc;
use core::ops::{Bound, RangeBounds};
pub use iter::{IntoIter, Iter};
pub use kasane_logic_derive::BitMask;
use node::{Decision, Node};
pub use summary::{BitMask, MinMax, NoSummary, Summary, ValueSet};

mod iter;
mod node;
mod summary;
#[cfg(test)]
mod tests;

/// [FlexId]に対して割り当てられている値`V`を管理するためのインデックス構造。
///
/// ## 数値
///
/// ```
/// use kasane_logic::FlexId;
/// use kasane_logic::spatial_id::collection::flex_tree_2::FlexTreeCore2;
///
/// let mut tree = FlexTreeCore2::<u32>::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), 35u32);
/// tree.insert(FlexId::new(18, 1, 18, 26, 18, 50).unwrap(), 80u32);
///
/// assert_eq!(tree.value_range(), Some((&35, &80)));
/// let hot = tree.filter_range(50..);
/// assert_eq!(hot.len(), 1);
/// ```
///
/// ## Enum
///
/// enum に `#[derive(BitMask)]` を付け、型パラメーターに [ValueSet] を指定する。
/// これにより内部的にビットマスクが張られてフィルタリングが高速化する
///
/// ```
/// use kasane_logic::FlexId;
/// use kasane_logic::spatial_id::collection::flex_tree_2::{BitMask, FlexTreeCore2, ValueSet};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, BitMask)]
/// enum Land {
///     Road,
///     Building,
///     Water,
/// }
///
/// let mut tree: FlexTreeCore2<Land, ValueSet<Land>> = FlexTreeCore2::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), Land::Road);
/// tree.insert(FlexId::new(20, 5, 20, 101, 20, 200).unwrap(), Land::Water);
///
/// assert!(tree.value_set().contains(Land::Water));
/// let obstacles = tree.filter_values([Land::Building, Land::Water].into_iter().collect());
/// assert_eq!(obstacles.len(), 1);
/// ```
///
/// ## 範囲と種類の両方で絞る
///
/// 2つの集計をタプルで組み合わせる。型が長くなるので型エイリアスを付けるとよい。
///
/// ```
/// use kasane_logic::FlexId;
/// use kasane_logic::spatial_id::collection::flex_tree_2::{
///     BitMask, FlexTreeCore2, MinMax, ValueSet,
/// };
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BitMask)]
/// enum Risk {
///     Low,
///     Mid,
///     High,
/// }
///
/// let mut tree = FlexTreeCore2::<Risk, (MinMax<Risk>, ValueSet<Risk>)>::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), Risk::Low);
/// tree.insert(FlexId::new(20, 5, 20, 101, 20, 200).unwrap(), Risk::High);
///
/// assert_eq!(tree.filter_range(Risk::Mid..).len(), 1);
/// assert_eq!(tree.filter_values(ValueSet::single(Risk::Low)).len(), 1);
/// ```
#[derive(Debug, PartialEq)]
pub struct FlexTreeCore2<V, S = MinMax<V>> {
    upper_root: Arc<Node<V, S>>,
    lower_root: Arc<Node<V, S>>,
}

impl<V, S> Clone for FlexTreeCore2<V, S> {
    fn clone(&self) -> Self {
        FlexTreeCore2 {
            upper_root: self.upper_root.clone(),
            lower_root: self.lower_root.clone(),
        }
    }
}

impl<V, S> Default for FlexTreeCore2<V, S> {
    fn default() -> Self {
        FlexTreeCore2 {
            upper_root: Node::empty(),
            lower_root: Node::empty(),
        }
    }
}

impl<V, S> FlexTreeCore2<V, S> {
    /// 値を持つ[FlexId]の数。
    pub fn len(&self) -> usize {
        self.upper_root.leaf_count() + self.lower_root.leaf_count()
    }

    pub fn is_empty(&self) -> bool {
        self.upper_root.is_empty() && self.lower_root.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, V, S> {
        Iter::new(self)
    }

    /// `target` が属する[Node]と、その[FlexId]を返す。
    /// 北半球と南半球の最初の分割用。
    fn root_for(&mut self, target: &FlexId) -> (&mut Arc<Node<V, S>>, FlexId) {
        if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        }
    }
}

impl<V: PartialEq, S: Summary<V>> FlexTreeCore2<V, S> {
    /// [FlexId]と値を挿入する。
    /// 既に値がある場合には上書きされる。
    pub fn insert(&mut self, target: FlexId, value: V) {
        let (root, root_flexid) = self.root_for(&target);
        let written = Node::only_at(&root_flexid, &target, value);
        // 上書きは、書き込む値を優先した和集合
        *root = Node::merge(&root_flexid, &written, root, &Node::union_rule);
    }

    /// `target` の領域を空にする。
    pub fn remove(&mut self, target: FlexId) {
        let (root, root_flexid) = self.root_for(&target);
        let removed = Node::<(), NoSummary>::only_at(&root_flexid, &target, ());
        *root = Node::merge(&root_flexid, root, &removed, &Node::difference_rule);
    }

    /// 和集合。両方に値がある場所は `self` の値を使う。
    pub fn union(&self, other: &Self) -> Self {
        self.merge(other, &Node::union_rule)
    }

    /// 積集合。`other` にも値がある場所だけ、`self` の値を残す。
    pub fn intersection<W: PartialEq, T: Summary<W>>(&self, other: &FlexTreeCore2<W, T>) -> Self {
        self.merge(other, &Node::intersection_rule)
    }

    /// 差集合。`other` に値がある場所を `self` から取り除く。
    pub fn difference<W: PartialEq, T: Summary<W>>(&self, other: &FlexTreeCore2<W, T>) -> Self {
        self.merge(other, &Node::difference_rule)
    }

    /// 木全体の値の集計。空なら [`None`]。
    pub fn summary(&self) -> Option<S> {
        match (self.upper_root.summary(), self.lower_root.summary()) {
            (Some(upper), Some(lower)) => Some(upper.merge(&lower)),
            (Some(only), None) | (None, Some(only)) => Some(only.into_owned()),
            (None, None) => None,
        }
    }

    /// 上下のルートどうしを `rule` で重ね合わせる。
    fn merge<W: PartialEq, T: Summary<W>>(
        &self,
        other: &FlexTreeCore2<W, T>,
        rule: &impl Fn(&Arc<Node<V, S>>, &Arc<Node<W, T>>) -> Option<Arc<Node<V, S>>>,
    ) -> Self {
        FlexTreeCore2 {
            upper_root: Node::merge(
                &FlexId::UPPER_MAX,
                &self.upper_root,
                &other.upper_root,
                rule,
            ),
            lower_root: Node::merge(
                &FlexId::LOWER_MAX,
                &self.lower_root,
                &other.lower_root,
                rule,
            ),
        }
    }

    /// 値が条件を満たす領域だけを残す。詳しくは [`Node::filter`]。
    fn filter(&self, classify: &impl Fn(&S) -> Decision, keep: &impl Fn(&V) -> bool) -> Self {
        FlexTreeCore2 {
            upper_root: Node::filter(&FlexId::UPPER_MAX, &self.upper_root, classify, keep),
            lower_root: Node::filter(&FlexId::LOWER_MAX, &self.lower_root, classify, keep),
        }
    }
}

impl<V: Ord, S: Summary<V> + AsRef<MinMax<V>>> FlexTreeCore2<V, S> {
    /// [FlexTreeCore2]全体に存在する値の範囲 `[min, max]` を返す。[FlexTreeCore2]が空なら [`None`]。
    pub fn value_range(&self) -> Option<(&V, &V)> {
        match (self.upper_root.value_range(), self.lower_root.value_range()) {
            (None, None) => None,
            (Some(u), None) => Some(u),
            (None, Some(l)) => Some(l),
            (Some((u_min, u_max)), Some((l_min, l_max))) => {
                Some((u_min.min(l_min), u_max.max(l_max)))
            }
        }
    }

    /// 全体に存在する値の最小値を返す。値がまだなければ [`None`]。
    pub fn min_value(&self) -> Option<&V> {
        self.value_range().map(|(min, _)| min)
    }

    /// 全体に存在する値の最大値を返す。値がまだなければ [`None`]。
    pub fn max_value(&self) -> Option<&V> {
        self.value_range().map(|(_, max)| max)
    }

    /// 値が指定した範囲 `range` に含まれる領域だけを残した[FlexTreeCore2]を作成する。
    pub fn filter_range<R: RangeBounds<V>>(&self, range: R) -> Self {
        let classify = |summary: &S| {
            let (min, max) = (summary.as_ref().min(), summary.as_ref().max());
            if is_disjoint(min, max, range.start_bound(), range.end_bound()) {
                Decision::DropAll
            } else if range.contains(min) && range.contains(max) {
                Decision::KeepAll
            } else {
                Decision::Descend
            }
        };
        self.filter(&classify, &|value| range.contains(value))
    }
}

impl<V: BitMask + PartialEq, S: Summary<V> + AsRef<ValueSet<V>>> FlexTreeCore2<V, S> {
    /// 木全体に現れる値の集合。空なら [`ValueSet::EMPTY`]。
    pub fn value_set(&self) -> ValueSet<V> {
        self.summary()
            .map_or(ValueSet::EMPTY, |summary| *summary.as_ref())
    }

    /// 値が `values` に含まれる領域だけを残した[FlexTreeCore2]を作成する。
    pub fn filter_values(&self, values: ValueSet<V>) -> Self {
        let classify = |summary: &S| {
            let present = summary.as_ref();
            if present.is_disjoint(&values) {
                Decision::DropAll
            } else if present.is_subset(&values) {
                Decision::KeepAll
            } else {
                Decision::Descend
            }
        };
        self.filter(&classify, &|value| values.contains(*value))
    }
}

/// 値 `min..=max` が範囲 `(start, end)` と交差しないなら true。
fn is_disjoint<V: Ord>(min: &V, max: &V, start: Bound<&V>, end: Bound<&V>) -> bool {
    let before_start = match start {
        Bound::Included(s) => max < s,
        Bound::Excluded(s) => max <= s,
        Bound::Unbounded => false,
    };
    let after_end = match end {
        Bound::Included(e) => min > e,
        Bound::Excluded(e) => min >= e,
        Bound::Unbounded => false,
    };
    before_start || after_end
}
