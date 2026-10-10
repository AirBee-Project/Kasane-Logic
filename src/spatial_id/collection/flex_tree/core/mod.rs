use crate::spatial_id::collection::flex_tree::time_reconstruct;
use crate::{AllowedIntervals, FlexId, RangeId, Side, SingleId, SpatialId};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::iter::from_fn;
use core::mem;
use core::ops::{Bound, RangeBounds};
pub use iter::{IntoIter, Iter};
pub use kasane_logic_derive::BitMask;
use node::{Decision, Node};
pub use ptr::SafeValue;
pub use summary::{BitMask, MinMax, NoSummary, Summary, ValueSet};
use view::View;

mod iter;
mod node;
pub mod ptr;
mod summary;
#[cfg(test)]
mod tests;
mod view;

/// [FlexId]に対して割り当てられている値`V`を管理するためのインデックス構造。
///
/// `S` は Branch が子孫の値についてキャッシュする [Summary]。どの [Summary] でも全ての操作が使え、
/// 値の範囲や種類で絞り込む操作だけが、その情報を持つ [Summary] で速くなる。
///
/// ## 数値
///
/// [MinMax] を指定すると、値の範囲での絞り込みが速くなる。
///
/// ```
/// use kasane_logic::spatial_id::collection::flex_tree::core::FlexTreeCore;
/// use kasane_logic::{FlexId, MinMax};
///
/// let mut tree = FlexTreeCore::<u32, MinMax<u32>>::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), 35u32);
/// tree.insert(FlexId::new(18, 1, 18, 26, 18, 50).unwrap(), 80u32);
///
/// assert_eq!(tree.value_range(), Some((&35, &80)));
/// let hot = tree.filter_range(50..);
/// assert_eq!(hot.count(), 1);
/// ```
///
/// ## Enum
///
/// enum に `#[derive(BitMask)]` を付け、型パラメーターに [ValueSet] を指定する。
/// これにより内部的にビットマスクが張られてフィルタリングが高速化する
///
/// ```
/// use kasane_logic::FlexId;
/// use kasane_logic::spatial_id::collection::flex_tree::core::FlexTreeCore;
/// use kasane_logic::{BitMask, ValueSet};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, BitMask)]
/// enum Land {
///     Road,
///     Building,
///     Water,
/// }
///
/// let mut tree: FlexTreeCore<Land, ValueSet<Land>> = FlexTreeCore::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), Land::Road);
/// tree.insert(FlexId::new(20, 5, 20, 101, 20, 200).unwrap(), Land::Water);
///
/// assert!(tree.value_set().contains(Land::Water));
/// let obstacles = tree.filter_values([Land::Building, Land::Water].into_iter().collect());
/// assert_eq!(obstacles.count(), 1);
/// ```
///
/// ## 範囲と種類の両方で絞る
///
/// 2つの Summary をタプルで組み合わせる。型が長くなるので型エイリアスを付けるとよい。
///
/// ```
/// use kasane_logic::FlexId;
/// use kasane_logic::spatial_id::collection::flex_tree::core::FlexTreeCore;
/// use kasane_logic::{BitMask, MinMax, ValueSet};
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BitMask)]
/// enum Risk {
///     Low,
///     Mid,
///     High,
/// }
///
/// let mut tree = FlexTreeCore::<Risk, (MinMax<Risk>, ValueSet<Risk>)>::default();
/// tree.insert(FlexId::new(20, 5, 20, 100, 20, 200).unwrap(), Risk::Low);
/// tree.insert(FlexId::new(20, 5, 20, 101, 20, 200).unwrap(), Risk::High);
///
/// assert_eq!(tree.filter_range(Risk::Mid..).count(), 1);
/// assert_eq!(tree.filter_values(ValueSet::single(Risk::Low)).count(), 1);
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct FlexTreeCore<V, S = NoSummary> {
    upper_root: Node<V, S>,
    lower_root: Node<V, S>,
}

impl<V: Clone, S> Clone for FlexTreeCore<V, S> {
    fn clone(&self) -> Self {
        FlexTreeCore {
            upper_root: self.upper_root.clone(),
            lower_root: self.lower_root.clone(),
        }
    }
}

impl<V, S> Default for FlexTreeCore<V, S> {
    fn default() -> Self {
        FlexTreeCore {
            upper_root: Node::Empty,
            lower_root: Node::Empty,
        }
    }
}

impl<V, S> FlexTreeCore<V, S> {
    /// 値を持つ[FlexId]の数。
    pub fn count(&self) -> usize {
        self.upper_root.count() + self.lower_root.count()
    }

    pub fn is_empty(&self) -> bool {
        self.upper_root.is_empty() && self.lower_root.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, V, S> {
        Iter::new(self)
    }

    /// 値を持つ[FlexId]の F/X/Y のズームレベルのうち、最も高いもの。空なら [`None`]。
    ///
    /// キャッシュを持たず、値を持つ全ての[FlexId]を走査する（O(n)）。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.iter()
            .map(|(id, _)| id.f_zoomlevel().max(id.x_zoomlevel()).max(id.y_zoomlevel()))
            .max()
    }

    /// 形と値はそのままに、Summary を `T` に付け替えた[FlexTreeCore]を作成する。
    ///
    /// 挿入し直さず、Branch の Summary だけを下から計算し直す（Node の数に比例）。
    pub fn with_summary<T: Summary<V>>(&self) -> FlexTreeCore<V, T>
    where
        V: Clone,
    {
        FlexTreeCore {
            upper_root: self.upper_root.with_summary(),
            lower_root: self.lower_root.with_summary(),
        }
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
            // 積むのは `t` と重なる Node だけ。Branch では `t` がどちらの半分にあるかが割った次元だけで決まるので、
            // 両方の子の領域を作って確かめずに済む
            let mut stack = self.new_stack();
            stack.retain(|(_, root)| root.intersects(&t));
            from_fn(move || {
                while let Some((node, this)) = stack.pop() {
                    match node {
                        Node::Empty => {}
                        Node::Leaf(value) => return Some((this.intersection(&t)?, value)),
                        Node::Branch(branch)
                            if t.zoomlevel_on(branch.dimension)
                                > this.zoomlevel_on(branch.dimension) =>
                        {
                            let side = this.side_toward(branch.dimension, &t);
                            let child = match side {
                                Side::Lower => &branch.lower,
                                Side::Upper => &branch.upper,
                            };
                            stack.push((child, this.split_on(branch.dimension, side).unwrap()));
                        }
                        node => stack.extend(
                            node.children(this)
                                .filter(|(_, region)| region.intersects(&t))
                                .rev(),
                        ),
                    }
                }
                None
            })
        })
    }

    /// `target` と重なる領域と値を、切り取らずにそのまま返す。各領域は1回だけ返す。
    pub fn get_overlapping<'a>(
        &'a self,
        target: impl IntoIterator<Item = FlexId>,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        let targets: Vec<FlexId> = target.into_iter().collect();
        self.overlapping(move |region| targets.iter().any(|t| region.intersects(t)))
    }

    /// `target` と面で接している領域と値を返す。`target` 自身と重なる領域は除く。
    pub fn neighbors_share_face<'a, T: SpatialId>(
        &'a self,
        target: T,
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

        let own: Vec<FlexId> = target.into_iter().collect();
        self.get_overlapping(shifted).filter(move |(leaf, _)| {
            own.iter().all(|o| !leaf.intersects(o)) && own.iter().any(|o| o.shares_face(leaf))
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
                stack.extend(node.children(this).rev());
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

impl<V: PartialEq, S> FlexTreeCore<V, S> {
    /// 時間方向に隣接する同値のSegmentを結合した [`RangeId`] と値を返す。**空間解像度は変えない**。
    ///
    /// `allowed_intervals` が [`None`] なら、各区間はそれを表せる最も粗い単位（`gcd(開始秒, 幅)`）の1Segmentになる。
    /// [`AllowedIntervals`] を渡すと、その候補のうち区間を割り切る最も粗い単位で表す。
    ///
    /// [`iter`](Self::iter) が返す [FlexId] は時間を2の冪秒のSegmentとして持つので、
    /// `1800` 秒のような単位で入れた ID も、こちらなら元の表記で取り出せる。
    pub fn reconstructed_time_ranges<'a>(
        &'a self,
        allowed_intervals: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + 'a {
        let has_temporal_split = (self.upper_root.split_dimensions()
            | self.lower_root.split_dimensions())
            & crate::Dimension::T.bit()
            != 0;
        time_reconstruct::reconstruct(self.iter(), has_temporal_split, allowed_intervals)
    }

    /// [`reconstructed_time_ranges`](Self::reconstructed_time_ranges) を、全体の最大ズームレベルに揃えた [`SingleId`] へ展開する。
    pub fn flat_single_ids<'a>(
        &'a self,
        allowed_intervals: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + 'a {
        let max_z = self.max_zoomlevel().unwrap_or(0);
        self.reconstructed_time_ranges(allowed_intervals)
            .flat_map(move |(range, value)| {
                let range = if range.z() == max_z {
                    range
                } else {
                    range
                        .spatial_children_at_zoom(max_z)
                        .expect("全体の最大ズームレベルは各Segmentのズーム以上")
                };
                range.single_ids().map(move |id| (id, value))
            })
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> FlexTreeCore<V, S> {
    /// `target` の領域に `value` を書き込む。既に値がある場所は上書きされる。
    ///
    /// `target` には [FlexId] のほか、[SingleId] や [RangeId] をそのまま渡せる。
    pub fn insert(&mut self, target: impl IntoIterator<Item = FlexId>, value: V) {
        self.insert_by_rule(target, value, &|this, existing, written| {
            if written.leaf().is_some() || existing.is_empty() {
                *existing = written.to_node(this);
                return true;
            }
            written.is_empty()
        });
    }

    /// `target` の領域に `value` を書き込む。既に値がある場所は `resolve(既存の値, value)` にする。
    pub fn insert_with(
        &mut self,
        target: impl IntoIterator<Item = FlexId>,
        value: V,
        resolve: impl Fn(&V, &V) -> V,
    ) {
        self.insert_by_rule(target, value, &|this, existing, written| {
            match (&*existing, written.leaf()) {
                _ if written.is_empty() => {}
                (Node::Empty, _) => *existing = written.to_node(this),
                (Node::Leaf(old), Some(new)) => *existing = Node::Leaf(resolve(old, new)),
                _ => return false,
            }
            true
        });
    }

    /// `target` の領域を空にし、取り除いた部分を FlexTreeCore として返す。
    ///
    /// 1回の降下で、`target` に覆われた Node をその領域ごと抜き取り、あとで取り除いた部分に組み直す。
    pub fn remove(&mut self, target: impl IntoIterator<Item = FlexId>) -> Self {
        let cut = RefCell::new(Vec::new());
        let whole = Node::<(), NoSummary>::Leaf(());
        let rule = |this: &FlexId, a: &mut Node<V, S>, b: View<'_, (), NoSummary>| {
            if b.leaf().is_some() {
                if !a.is_empty() {
                    cut.borrow_mut().push((*this, mem::take(a)));
                }
                return true;
            }
            a.is_empty() || b.is_empty()
        };
        for id in target {
            let (root, root_id) = self.root_for(&id);
            Node::merge(&root_id, root, View::skip(&root_id, id, &whole), &rule);
        }

        // 抜き取った部分は互いに重ならないので、和で順に植え直すだけでよい
        let mut removed = Self::default();
        for (region, node) in cut.into_inner() {
            let (root, root_id) = removed.root_for(&region);
            Node::merge(
                &root_id,
                root,
                View::skip(&root_id, region, &node),
                &Node::union_rule,
            );
        }
        removed
    }

    /// `target` と重なる領域を、切り取らずに丸ごと取り除き、取り除いた部分を FlexTreeCore として返す。
    pub fn remove_overlapping(&mut self, target: impl IntoIterator<Item = FlexId>) -> Self {
        let targets: Vec<FlexId> = target.into_iter().collect();
        let overlaps = |region: &FlexId| targets.iter().any(|t| region.intersects(t));
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
        let mut result = self.clone();
        result.merge(other, &Node::union_rule);
        result
    }

    /// 積集合。`other` にも値がある場所だけ、`self` の値を残す。
    pub fn intersection<W: PartialEq, T: Summary<W>>(&self, other: &FlexTreeCore<W, T>) -> Self {
        let mut result = self.clone();
        result.merge(other, &Node::intersection_rule);
        result
    }

    /// 差集合。`other` に値がある場所を `self` から取り除く。
    pub fn difference<W: PartialEq, T: Summary<W>>(&self, other: &FlexTreeCore<W, T>) -> Self {
        let mut result = self.clone();
        result.merge(other, &Node::difference_rule);
        result
    }

    /// FlexTreeCore 全体の Summary。空なら [`None`]。
    pub fn summary(&self) -> Option<S> {
        match (self.upper_root.summary(), self.lower_root.summary()) {
            (Some(upper), Some(lower)) => Some(upper.merge(&lower)),
            (Some(only), None) | (None, Some(only)) => Some(only.into_owned()),
            (None, None) => None,
        }
    }

    /// `target` の各領域だけに `value` がある Node を、既存の Node へ `rule` で重ね合わせる。
    fn insert_by_rule(
        &mut self,
        target: impl IntoIterator<Item = FlexId>,
        value: V,
        rule: &impl Fn(&FlexId, &mut Node<V, S>, View<'_, V, S>) -> bool,
    ) {
        let leaf = Node::Leaf(value);
        for id in target {
            let (root, root_id) = self.root_for(&id);
            Node::merge(&root_id, root, View::skip(&root_id, id, &leaf), rule);
        }
    }

    /// 上下のルートへ、`other` のルートを `rule` で重ね合わせる。
    fn merge<W, T>(
        &mut self,
        other: &FlexTreeCore<W, T>,
        rule: &impl Fn(&FlexId, &mut Node<V, S>, View<'_, W, T>) -> bool,
    ) {
        let upper = View::from(&other.upper_root);
        let lower = View::from(&other.lower_root);
        Node::merge(&FlexId::UPPER_MAX, &mut self.upper_root, upper, rule);
        Node::merge(&FlexId::LOWER_MAX, &mut self.lower_root, lower, rule);
    }

    /// 条件を満たす[FlexId]だけを残す。
    fn filter(
        &self,
        classify: &impl Fn(&FlexId, &S) -> Decision,
        keep: &impl Fn(&FlexId, &V) -> bool,
    ) -> Self {
        let filter = |root: &Node<V, S>, this: &FlexId| {
            root.filter(this, classify, keep)
                .unwrap_or_else(|| root.clone())
        };
        FlexTreeCore {
            upper_root: filter(&self.upper_root, &FlexId::UPPER_MAX),
            lower_root: filter(&self.lower_root, &FlexId::LOWER_MAX),
        }
    }
}

impl<V: Ord + Clone, S: Summary<V>> FlexTreeCore<V, S> {
    /// [FlexTreeCore]全体に存在する値の範囲 `(最小, 最大)` を返す。[FlexTreeCore]が空なら [`None`]。
    ///
    /// 値の範囲を持つ Summary（[MinMax]）なら O(1)、そうでなければ全ての値を走査する。
    pub fn value_range(&self) -> Option<(&V, &V)> {
        node::wider_range(self.upper_root.value_range(), self.lower_root.value_range())
    }

    /// 値が指定した範囲 `range` に含まれる領域だけを残した[FlexTreeCore]を作成する。
    ///
    /// 値の範囲を持つ Summary（[MinMax]）なら、子孫がすべて範囲の内側・外側にある部分は辿らない。
    pub fn filter_range<R: RangeBounds<V>>(&self, range: R) -> Self {
        let classify = |_: &FlexId, summary: &S| match summary.value_range() {
            Some((min, max)) if is_disjoint(min, max, range.start_bound(), range.end_bound()) => {
                Decision::DropAll
            }
            Some((min, max)) if range.contains(min) && range.contains(max) => Decision::KeepAll,
            _ => Decision::Descend,
        };
        self.filter(&classify, &|_, value| range.contains(value))
    }
}

impl<V: BitMask + PartialEq + Clone, S: Summary<V>> FlexTreeCore<V, S> {
    /// FlexTreeCore 全体に現れる値の集合。空なら [`ValueSet::EMPTY`]。
    ///
    /// 値の集合を持つ Summary（[ValueSet]）なら O(1)、そうでなければ全ての値を走査する。
    pub fn value_set(&self) -> ValueSet<V> {
        self.upper_root
            .value_set()
            .union(&self.lower_root.value_set())
    }

    /// 値が `values` に含まれる領域だけを残した[FlexTreeCore]を作成する。
    ///
    /// 値の集合を持つ Summary（[ValueSet]）なら、子孫がすべて `values` の内側・外側にある部分は辿らない。
    pub fn filter_values(&self, values: ValueSet<V>) -> Self {
        let classify = |_: &FlexId, summary: &S| match summary.value_set() {
            Some(present) if present.is_disjoint(&values) => Decision::DropAll,
            Some(present) if present.is_subset(&values) => Decision::KeepAll,
            _ => Decision::Descend,
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
