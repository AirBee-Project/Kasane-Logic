use crate::{FlexId, RangeId, SpatialId};
use alloc::vec::Vec;
use core::iter::from_fn;
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
/// 2つの Summary をタプルで組み合わせる。型が長くなるので型エイリアスを付けるとよい。
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
#[derive(Debug, PartialEq, Eq)]
pub struct FlexTreeCore2<V, S = MinMax<V>> {
    upper_root: Node<V, S>,
    lower_root: Node<V, S>,
}

impl<V: Clone, S> Clone for FlexTreeCore2<V, S> {
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
            upper_root: Node::Empty,
            lower_root: Node::Empty,
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

    /// 全ての値を消す。
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// `target` と重なる領域と値を、`target` との共通部分に切り取って返す。
    ///
    /// `target` の各[FlexId]が互いに重ならなければ、返す領域も互いに重ならない。
    pub fn get<'a>(
        &'a self,
        target: impl IntoIterator<Item = FlexId, IntoIter: 'a>,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        target.into_iter().flat_map(move |t| {
            self.overlapping(move |region| region.intersection(&t).is_some())
                .filter_map(move |(leaf, value)| Some((leaf.intersection(&t)?, value)))
        })
    }

    /// `target` と重なる領域と値を、切り取らずにそのまま返す。各領域は1回だけ返す。
    pub fn get_overlapping<'a>(
        &'a self,
        target: impl IntoIterator<Item = FlexId>,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let targets: Vec<FlexId> = target.into_iter().collect();
        self.overlapping(move |region| targets.iter().any(|t| region.intersection(t).is_some()))
    }

    /// 範囲 `target` と重なる領域と値を、切り取らずにそのまま返す。時間軸も含めて判定する。
    pub fn get_overlapping_range<'a>(
        &'a self,
        target: &RangeId,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let target = target.clone();
        self.overlapping(move |region| region.intersects_range(&target))
    }

    /// `target` と面で接している領域と値を返す。`target` 自身と重なる領域は除く。
    pub fn neighbors_share_face<'a, T: SpatialId>(
        &'a self,
        target: &T,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        // 各方向に1つずらした領域が、面で接しうる候補
        let mut shifted: Vec<FlexId> = Vec::new();
        for delta in [-1, 1] {
            let mut f = target.clone();
            if f.move_f(delta).is_ok() {
                shifted.extend(f);
            }
            let mut y = target.clone();
            if y.move_y(delta).is_ok() {
                shifted.extend(y);
            }
            let mut x = target.clone();
            x.move_x(delta);
            shifted.extend(x);
        }

        let own: Vec<FlexId> = target.clone().into_iter().collect();
        self.get_overlapping(shifted).filter(move |(leaf, _)| {
            own.iter().all(|o| leaf.intersection(o).is_none())
                && own.iter().any(|o| o.shares_face(leaf))
        })
    }

    /// 値を持つ Leaf のうち、領域が `overlaps` を満たすものを FlexId と値で返す。`overlaps` を満たさない Node の子孫は辿らない。
    fn overlapping<'a>(
        &'a self,
        overlaps: impl Fn(&FlexId) -> bool + 'a,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let mut stack = self.new_stack();
        from_fn(move || {
            while let Some((node, this)) = stack.pop() {
                if !overlaps(&this) {
                    continue;
                }
                if let Node::Leaf(value) = node {
                    return Some((this, value));
                }
                node.push_children(this, &mut stack);
            }
            None
        })
    }

    /// 上下のルートを積んだ、[Node] を辿るためのスタックを返す。
    fn new_stack(&self) -> Vec<(&Node<V, S>, FlexId)> {
        // 辿っている経路の各 Branch につき未処理の子が1つ積まれ、もう一方のルートと今の子の分が加わる。
        // スタックは高さ + 2 を超えないので、最後まで再確保しない。
        let height = self.upper_root.height().max(self.lower_root.height());
        let mut stack = Vec::with_capacity(usize::from(height) + 2);
        // 後に積んだものから取り出すので、上側のルートを後に積む
        stack.push((&self.lower_root, FlexId::LOWER_MAX));
        stack.push((&self.upper_root, FlexId::UPPER_MAX));
        stack
    }

    /// `target` を含む方のルート（北半球か南半球）と、その領域を返す。
    fn root_for(&mut self, target: &FlexId) -> (&mut Node<V, S>, FlexId) {
        if target.f_index().is_negative() {
            (&mut self.lower_root, FlexId::LOWER_MAX)
        } else {
            (&mut self.upper_root, FlexId::UPPER_MAX)
        }
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> FlexTreeCore2<V, S> {
    /// `target` の領域に `value` を書き込む。既に値がある場所は上書きされる。
    ///
    /// `target` には [FlexId] のほか、[SingleId](crate::SingleId) や [RangeId] をそのまま渡せる。
    pub fn insert(&mut self, target: impl IntoIterator<Item = FlexId>, value: V) {
        // 上書きは、書き込む値を優先した和集合
        self.insert_by_rule(target, value, &|existing, written| {
            Node::union_rule(written, existing)
        });
    }

    /// `target` の領域に `value` を書き込む。既に値がある場所は `resolve(既存の値, value)` にする。
    pub fn insert_with(
        &mut self,
        target: impl IntoIterator<Item = FlexId>,
        value: V,
        resolve: impl Fn(&V, &V) -> V,
    ) {
        self.insert_by_rule(
            target,
            value,
            &|existing, written| match (existing, written) {
                (_, Node::Empty) => Some(existing.clone()),
                (Node::Empty, _) => Some(written.clone()),
                (Node::Leaf(old), Node::Leaf(new)) => Some(Node::Leaf(resolve(old, new))),
                _ => None,
            },
        );
    }

    /// `target` の領域を空にし、取り除いた部分を FlexTreeCore2 として返す。
    pub fn remove(&mut self, target: impl IntoIterator<Item = FlexId>) -> Self {
        let mut region = FlexTreeCore2::<(), NoSummary>::default();
        region.insert(target, ());
        let removed = self.intersection(&region);
        *self = self.difference(&region);
        removed
    }

    /// `target` と重なる領域を、切り取らずに丸ごと取り除き、取り除いた部分を FlexTreeCore2 として返す。
    pub fn remove_overlapping(&mut self, target: impl IntoIterator<Item = FlexId>) -> Self {
        let targets: Vec<FlexId> = target.into_iter().collect();
        let overlaps = |region: &FlexId| targets.iter().any(|t| region.intersection(t).is_some());
        let removed_part = |region: &FlexId, _: &S| {
            if !overlaps(region) {
                Decision::DropAll
            } else if targets.iter().any(|t| t.contains(region)) {
                Decision::KeepAll
            } else {
                Decision::Descend
            }
        };
        let removed = self.filter(&removed_part, &|leaf, _| overlaps(leaf));
        *self = self.filter(
            &|region, summary| match removed_part(region, summary) {
                Decision::KeepAll => Decision::DropAll,
                Decision::DropAll => Decision::KeepAll,
                Decision::Descend => Decision::Descend,
            },
            &|leaf, _| !overlaps(leaf),
        );
        removed
    }

    /// 和集合。両方に値がある場所は `self` の値を使う。
    pub fn union(&self, other: &Self) -> Self {
        self.merge(other, &Node::union_rule)
    }

    /// 積集合。`other` にも値がある場所だけ、`self` の値を残す。
    pub fn intersection<W: PartialEq + Clone, T: Summary<W>>(
        &self,
        other: &FlexTreeCore2<W, T>,
    ) -> Self {
        self.merge(other, &Node::intersection_rule)
    }

    /// 差集合。`other` に値がある場所を `self` から取り除く。
    pub fn difference<W: PartialEq + Clone, T: Summary<W>>(
        &self,
        other: &FlexTreeCore2<W, T>,
    ) -> Self {
        self.merge(other, &Node::difference_rule)
    }

    /// FlexTreeCore2 全体の Summary。空なら [`None`]。
    pub fn summary(&self) -> Option<S> {
        match (self.upper_root.summary(), self.lower_root.summary()) {
            (Some(upper), Some(lower)) => Some(upper.merge(&lower)),
            (Some(only), None) | (None, Some(only)) => Some(only.into_owned()),
            (None, None) => None,
        }
    }

    /// `target` の各領域だけに `value` を持つ Node を作り、既存の Node と `rule` で重ね合わせる。
    fn insert_by_rule(
        &mut self,
        target: impl IntoIterator<Item = FlexId>,
        value: V,
        rule: &impl Fn(&Node<V, S>, &Node<V, S>) -> Option<Node<V, S>>,
    ) {
        let leaf = Node::Leaf(value);
        for id in target {
            let (root, root_id) = self.root_for(&id);
            let written = Node::skip(&root_id, &id, leaf.clone());
            *root = Node::merge(&root_id, root, &written, rule);
        }
    }

    /// 上下のルートどうしを `rule` で重ね合わせる。
    fn merge<W: PartialEq + Clone, T: Summary<W>>(
        &self,
        other: &FlexTreeCore2<W, T>,
        rule: &impl Fn(&Node<V, S>, &Node<W, T>) -> Option<Node<V, S>>,
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

    /// 条件を満たす[FlexId]だけを残す。
    fn filter(
        &self,
        classify: &impl Fn(&FlexId, &S) -> Decision,
        keep: &impl Fn(&FlexId, &V) -> bool,
    ) -> Self {
        FlexTreeCore2 {
            upper_root: Node::filter(&FlexId::UPPER_MAX, &self.upper_root, classify, keep),
            lower_root: Node::filter(&FlexId::LOWER_MAX, &self.lower_root, classify, keep),
        }
    }
}

impl<V: Ord + Clone, S: Summary<V> + AsRef<MinMax<V>>> FlexTreeCore2<V, S> {
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
        let classify = |_: &FlexId, summary: &S| {
            let (min, max) = (summary.as_ref().min(), summary.as_ref().max());
            if is_disjoint(min, max, range.start_bound(), range.end_bound()) {
                Decision::DropAll
            } else if range.contains(min) && range.contains(max) {
                Decision::KeepAll
            } else {
                Decision::Descend
            }
        };
        self.filter(&classify, &|_, value| range.contains(value))
    }
}

impl<V: BitMask + PartialEq, S: Summary<V> + AsRef<ValueSet<V>>> FlexTreeCore2<V, S> {
    /// FlexTreeCore2 全体に現れる値の集合。空なら [`ValueSet::EMPTY`]。
    pub fn value_set(&self) -> ValueSet<V> {
        self.summary()
            .map_or(ValueSet::EMPTY, |summary| *summary.as_ref())
    }

    /// 値が `values` に含まれる領域だけを残した[FlexTreeCore2]を作成する。
    pub fn filter_values(&self, values: ValueSet<V>) -> Self {
        let classify = |_: &FlexId, summary: &S| {
            let present = summary.as_ref();
            if present.is_disjoint(&values) {
                Decision::DropAll
            } else if present.is_subset(&values) {
                Decision::KeepAll
            } else {
                Decision::Descend
            }
        };
        self.filter(&classify, &|_, value| values.contains(*value))
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
