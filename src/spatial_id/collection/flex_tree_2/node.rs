use alloc::borrow::Cow;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ptr;

use super::summary::{MinMax, Summary};
use crate::{
    FlexId, Side,
    spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId},
};

/// Node は自分の領域を持たず、親から渡される領域 `this`との相対的な位置で意味を持つ。辿る関数が`this`を持つことで様々な操作を行う。
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Node<V, S> {
    Empty,
    Leaf(V),
    Branch(Arc<Branch<V, S>>),
    Skip(Arc<Skip<V, S>>),
}

impl<V: Clone, S> Clone for Node<V, S> {
    fn clone(&self) -> Self {
        match self {
            Node::Empty => Node::Empty,
            Node::Leaf(value) => Node::Leaf(value.clone()),
            Node::Branch(branch) => Node::Branch(branch.clone()),
            Node::Skip(skip) => Node::Skip(skip.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Branch<V, S> {
    pub(super) dimension: Dimension,
    /// 自身と子孫が割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
    pub(super) split_dimensions: u8,
    /// 自身と子孫で Branch が縦に何段続くかの最大。Node を辿るスタックの大きさに使う。
    pub(super) height: u8,
    /// 子孫に含まれる、値を持つ[FlexId]の数。
    pub(super) leaf_count: usize,
    pub(super) summary: S,
    pub(super) lower: Node<V, S>,
    pub(super) upper: Node<V, S>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Skip<V, S> {
    pub(super) path: RelativeFlexId,
    /// 自身と子孫が割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
    pub(super) split_dimensions: u8,
    pub(super) child: Node<V, S>,
}

/// [`Node::filter`] で、Branch の領域と Summary から子孫をまとめて判断した結果。
pub(super) enum Decision {
    /// 子孫をすべて残す。
    KeepAll,
    /// 子孫をすべて捨てる。
    DropAll,
    /// 子へ降りて判断する。
    Descend,
}

impl<V, S> Node<V, S> {
    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }

    /// 子孫に含まれる、値を持つ[FlexId]の数。
    pub(super) fn leaf_count(&self) -> usize {
        match self {
            Node::Empty => 0,
            Node::Leaf(_) => 1,
            Node::Branch(branch) => branch.leaf_count,
            Node::Skip(skip) => skip.child.leaf_count(),
        }
    }

    /// 自身と子孫で Branch が縦に何段続くかの最大。Leaf なら 0。
    pub(super) fn height(&self) -> u8 {
        match self {
            Node::Empty | Node::Leaf(_) => 0,
            Node::Branch(branch) => branch.height,
            Node::Skip(skip) => skip.child.height(),
        }
    }

    /// 自身と子孫が割っている次元の集合のビットマスク。ビットマスクは[`Dimension::bit`]のOR。
    pub(super) fn split_dimensions(&self) -> u8 {
        match self {
            Node::Empty | Node::Leaf(_) => 0,
            Node::Branch(branch) => branch.split_dimensions,
            Node::Skip(skip) => skip.split_dimensions,
        }
    }

    /// この Node を「最初にどの次元で割るか」。Leaf なら [`None`]。
    fn head_dimension(&self, this: &FlexId) -> Option<Dimension> {
        match self {
            Node::Empty | Node::Leaf(_) => None,
            Node::Branch(branch) => Some(branch.dimension),
            Node::Skip(skip) => this.coarsest_dimension_in(skip.split_dimensions),
        }
    }

    /// 領域 `this` のこの Node の子を、下側から取り出されるように `stack` へ積む。Leaf なら何もしない。
    pub(super) fn push_children<'a>(&'a self, this: FlexId, stack: &mut Vec<(&'a Self, FlexId)>) {
        match self {
            Node::Empty | Node::Leaf(_) => {}
            Node::Branch(branch) => {
                let dimension = branch.dimension;
                stack.push((
                    &branch.upper,
                    this.split_on(dimension, Side::Upper).unwrap(),
                ));
                stack.push((
                    &branch.lower,
                    this.split_on(dimension, Side::Lower).unwrap(),
                ));
            }
            Node::Skip(skip) => {
                stack.push((&skip.child, skip.path.to_absolute(&this).unwrap()));
            }
        }
    }

    /// 同じ Branch・Skip の Arc を指しているなら true。値の型が違う Node とも比べられる。
    fn ptr_eq<W, T>(&self, other: &Node<W, T>) -> bool {
        match (self, other) {
            (Node::Branch(a), Node::Branch(b)) => ptr::addr_eq(Arc::as_ptr(a), Arc::as_ptr(b)),
            (Node::Skip(a), Node::Skip(b)) => ptr::addr_eq(Arc::as_ptr(a), Arc::as_ptr(b)),
            _ => false,
        }
    }
}

impl<V, S: AsRef<MinMax<V>>> Node<V, S> {
    /// この Node と子孫の値の範囲 `[min, max]` を返す。空なら [`None`]。
    pub(super) fn value_range(&self) -> Option<(&V, &V)> {
        match self {
            Node::Empty => None,
            Node::Leaf(v) => Some((v, v)),
            Node::Branch(branch) => {
                let range = branch.summary.as_ref();
                Some((range.min(), range.max()))
            }
            Node::Skip(skip) => skip.child.value_range(),
        }
    }
}

impl<V, S: Summary<V>> Node<V, S> {
    /// この Node と子孫の値の Summary。空なら [`None`]。
    pub(super) fn summary(&self) -> Option<Cow<'_, S>> {
        match self {
            Node::Empty => None,
            Node::Leaf(value) => Some(Cow::Owned(S::new(value))),
            Node::Branch(branch) => Some(Cow::Borrowed(&branch.summary)),
            Node::Skip(skip) => skip.child.summary(),
        }
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> Node<V, S> {
    /// 中身を辿らずに同じと分かるなら true。変化の無い部分で元の Arc を使い回すために使う。
    fn is_same(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Empty, Node::Empty) => true,
            (Node::Leaf(a), Node::Leaf(b)) => a == b,
            _ => self.ptr_eq(other),
        }
    }

    /// 自身が `dimension` で割った Branch で、子が `lower`・`upper` と同じなら true。
    fn has_children(&self, dimension: Dimension, lower: &Self, upper: &Self) -> bool {
        matches!(self, Node::Branch(branch)
            if branch.dimension == dimension && branch.lower.is_same(lower) && branch.upper.is_same(upper))
    }

    /// 領域 `this` を `dimension` で割った左右 `lower`,`upper` を、カノニカル形の1つの[Node]に合わせる。
    fn join(this: &FlexId, dimension: Dimension, lower: Self, upper: Self) -> Self {
        // Branch はここでしか作らない。これでどの操作の結果もカノニカル形になる。
        // Node は領域との相対位置で意味を持つので、子孫が `dimension` で割っている Node を
        // 広い `this` へ持ち上げると意味が変わる。左右が同じでも、そのときは割ったままにする。
        if lower.split_dimensions() & dimension.bit() == 0
            && (lower.is_same(&upper) || lower == upper)
        {
            return lower;
        }
        // カノニカル形では Branch の子は空にならない
        if lower.is_empty() {
            let upper_id = this.split_on(dimension, Side::Upper).unwrap();
            return Node::skip(this, &upper_id, upper);
        }
        if upper.is_empty() {
            let lower_id = this.split_on(dimension, Side::Lower).unwrap();
            return Node::skip(this, &lower_id, lower);
        }
        let (Some(lower_summary), Some(upper_summary)) = (lower.summary(), upper.summary()) else {
            unreachable!("空でないNodeは必ずSummaryを持つ")
        };
        let summary = lower_summary.merge(&upper_summary);
        Node::Branch(Arc::new(Branch {
            dimension,
            split_dimensions: dimension.bit() | lower.split_dimensions() | upper.split_dimensions(),
            height: 1 + lower.height().max(upper.height()),
            leaf_count: lower.leaf_count() + upper.leaf_count(),
            summary,
            lower,
            upper,
        }))
    }

    /// 領域 `this` のうち、内側の `target` だけに `child` があり、外側は空の[Node]を作る。
    pub(super) fn skip(this: &FlexId, target: &FlexId, child: Self) -> Self {
        if target == this {
            return child;
        }
        let (target_id, child) = match child {
            // 空はどこに置いても空
            Node::Empty => return Node::Empty,
            // Skip の先へさらに Skip しないよう、Skip の行き先から直接張り直す
            Node::Skip(skip) => (skip.path.to_absolute(target).unwrap(), skip.child.clone()),
            child => (*target, child),
        };
        let path = target_id.relative_to(this).unwrap();
        let split_dimensions = path.deeper_dimensions() | child.split_dimensions();
        Node::Skip(Arc::new(Skip {
            path,
            split_dimensions,
            child,
        }))
    }

    /// 2 つの Node `a`・`b` を、領域 `this` の上で `merge_rule` に従って重ね合わせる。
    pub(super) fn merge<W: PartialEq + Clone, T: Summary<W>>(
        this: &FlexId,
        a: &Self,
        b: &Node<W, T>,
        merge_rule: &impl Fn(&Self, &Node<W, T>) -> Option<Self>,
    ) -> Self {
        if let Some(result) = merge_rule(a, b).or_else(|| Node::merge_skips(this, a, b, merge_rule))
        {
            return result;
        }

        // 結果もカノニカル形（粗い次元から割る）になるよう、両方の最初の次元のうち粗い方で割る
        let heads = a.head_dimension(this).map_or(0, |d| d.bit())
            | b.head_dimension(this).map_or(0, |d| d.bit());
        let dimension = this
            .coarsest_dimension_in(heads)
            .expect("Leaf どうしは merge_rule が答えを決める");

        let [a_lower, a_upper] = a.split(this, dimension);
        let [b_lower, b_upper] = b.split(this, dimension);
        let lower_id = this.split_on(dimension, Side::Lower).unwrap();
        let upper_id = this.split_on(dimension, Side::Upper).unwrap();
        let lower = Node::merge(&lower_id, &a_lower, &b_lower, merge_rule);
        let upper = Node::merge(&upper_id, &a_upper, &b_upper, merge_rule);

        if a.has_children(dimension, &lower, &upper) {
            return a.clone();
        }
        Node::join(this, dimension, lower, upper)
    }

    /// `a`・`b` がどちらも Skip で、2 つの行き先を共に含む `this` より狭い領域があれば、
    /// その領域で重ね合わせた結果を返す。無ければ [`None`]。
    fn merge_skips<W: PartialEq + Clone, T: Summary<W>>(
        this: &FlexId,
        a: &Self,
        b: &Node<W, T>,
        rule: &impl Fn(&Self, &Node<W, T>) -> Option<Self>,
    ) -> Option<Self> {
        let (Node::Skip(skip_a), Node::Skip(skip_b)) = (a, b) else {
            return None;
        };
        // 行き先が分かれるまでは途中の Node を作らずに降り、分かれる地点で1回だけ重ね合わせる
        let region_a = skip_a.path.to_absolute(this).unwrap();
        let region_b = skip_b.path.to_absolute(this).unwrap();
        let (split_a, split_b) = (
            skip_a.child.split_dimensions(),
            skip_b.child.split_dimensions(),
        );

        let mut at = *this;
        while let (Some(da), Some(db)) = (
            skip_head(&at, &region_a, split_a),
            skip_head(&at, &region_b, split_b),
        ) {
            if da != db {
                break;
            }
            let next = at.split_toward(da, &region_a).unwrap();
            if !next.contains(&region_b) {
                break;
            }
            at = next;
        }
        if at == *this {
            return None;
        }

        let a = Node::skip(&at, &region_a, skip_a.child.clone());
        let b = Node::skip(&at, &region_b, skip_b.child.clone());
        Some(Node::skip(this, &at, Node::merge(&at, &a, &b, rule)))
    }

    /// 領域 `this` のこの Node を `dimension` で割った `[下, 上]`（`join` の逆）。
    ///
    /// `dimension` で割っていなければ、両側にこの Node を返す。
    fn split(&self, this: &FlexId, dimension: Dimension) -> [Cow<'_, Self>; 2] {
        match self {
            Node::Branch(branch) if branch.dimension == dimension => {
                [Cow::Borrowed(&branch.lower), Cow::Borrowed(&branch.upper)]
            }
            Node::Skip(skip) if skip.path.depth_on(dimension) > 0 => {
                let region = skip.path.to_absolute(this).unwrap();
                let next = this.split_toward(dimension, &region).unwrap();
                let rest = Cow::Owned(Node::skip(&next, &region, skip.child.clone()));
                if next == this.split_on(dimension, Side::Upper).unwrap() {
                    [Cow::Owned(Node::Empty), rest]
                } else {
                    [rest, Cow::Owned(Node::Empty)]
                }
            }
            // その次元では中身が変わらないので、下も上も同じ中身になる
            _ => [Cow::Borrowed(self), Cow::Borrowed(self)],
        }
    }

    /// 和。`a` に値がある場所は `a`、無い場所は `b`。
    pub(super) fn union_rule(a: &Self, b: &Self) -> Option<Self> {
        match (a, b) {
            _ if a.ptr_eq(b) => Some(a.clone()),
            (Node::Leaf(_), _) | (_, Node::Empty) => Some(a.clone()),
            (Node::Empty, _) => Some(b.clone()),
            _ => None,
        }
    }

    /// 積。`b` に値がある場所だけ `a` を残す。
    pub(super) fn intersection_rule<W, T>(a: &Self, b: &Node<W, T>) -> Option<Self> {
        match (a, b) {
            _ if a.ptr_eq(b) => Some(a.clone()),
            (Node::Empty, _) | (_, Node::Leaf(_)) => Some(a.clone()),
            (_, Node::Empty) => Some(Node::Empty),
            _ => None,
        }
    }

    /// 差。`b` に値がある場所の `a` を消す。
    pub(super) fn difference_rule<W, T>(a: &Self, b: &Node<W, T>) -> Option<Self> {
        match (a, b) {
            _ if a.ptr_eq(b) => Some(Node::Empty),
            (Node::Empty, _) | (_, Node::Empty) => Some(a.clone()),
            (_, Node::Leaf(_)) => Some(Node::Empty),
            _ => None,
        }
    }

    /// 領域 `this` の `node` から、条件を満たす Leaf だけを残す。
    ///
    /// Branch は領域と Summary を `classify` で判断し、子孫をまとめて残すか捨てられるならそれ以上降りない。
    /// Leaf は領域と値を `keep` で判断する。変化の無い Node は元の [Arc] をそのまま使う。
    pub(super) fn filter(
        this: &FlexId,
        node: &Self,
        classify: &impl Fn(&FlexId, &S) -> Decision,
        keep: &impl Fn(&FlexId, &V) -> bool,
    ) -> Self {
        match node {
            Node::Empty => Node::Empty,
            Node::Leaf(value) if keep(this, value) => node.clone(),
            Node::Leaf(_) => Node::Empty,
            Node::Branch(branch) => match classify(this, &branch.summary) {
                Decision::KeepAll => node.clone(),
                Decision::DropAll => Node::Empty,
                Decision::Descend => {
                    let dimension = branch.dimension;
                    let lower_id = this.split_on(dimension, Side::Lower).unwrap();
                    let upper_id = this.split_on(dimension, Side::Upper).unwrap();
                    let lower = Node::filter(&lower_id, &branch.lower, classify, keep);
                    let upper = Node::filter(&upper_id, &branch.upper, classify, keep);
                    if node.has_children(dimension, &lower, &upper) {
                        return node.clone();
                    }
                    Node::join(this, dimension, lower, upper)
                }
            },
            Node::Skip(skip) => {
                let region = skip.path.to_absolute(this).unwrap();
                let child = Node::filter(&region, &skip.child, classify, keep);
                if child.is_same(&skip.child) {
                    return node.clone();
                }
                Node::skip(this, &region, child)
            }
        }
    }
}

/// Skip の行き先が `region`、その先で割っている次元が `child_split` のとき、
/// 領域 `at` で最初に割る次元。`at` が行き先そのものなら [`None`]。
fn skip_head(at: &FlexId, region: &FlexId, child_split: u8) -> Option<Dimension> {
    let finer = region.finer_dimensions_than(at);
    if finer == 0 {
        return None;
    }
    let dimension = at.coarsest_dimension_in(finer | child_split)?;
    debug_assert!(
        finer & dimension.bit() != 0,
        "カノニカルな Skip は、行き先が狭い次元から割る"
    );
    Some(dimension)
}
