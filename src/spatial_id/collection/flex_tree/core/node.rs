use super::summary::{MinMax, Summary};
use super::view::View;
use crate::{
    FlexId, Side,
    spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId},
};
use alloc::borrow::Cow;
use alloc::sync::Arc;
use core::{mem, ptr};

/// Node は自分の領域を持たず、親から渡される領域 `this`との相対的な位置で意味を持つ。辿る関数が`this`を持つことで様々な操作を行う。
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum Node<V, S> {
    #[default]
    Empty,
    Leaf(V),
    Branch(Arc<Branch<V, S>>),
    Skip(Arc<Skip<V, S>>),
}

// derive では `S: Clone` まで要求されるが、Arc の複製に `S` の複製は要らない
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Branch<V, S> {
    pub(super) dimension: Dimension,
    /// 自身と子孫が割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
    pub(super) split_dimensions: u8,
    /// 自身と子孫で Branch が縦に何段続くかの最大。Node を辿るスタックの大きさに使う。
    pub(super) height: u8,
    /// 子孫に含まれる、値を持つ[FlexId]の数。
    pub(super) count: usize,
    pub(super) summary: S,
    pub(super) lower: Node<V, S>,
    pub(super) upper: Node<V, S>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Skip<V, S> {
    pub(super) path: RelativeFlexId,
    /// 自身と子孫が割っている次元のビットマスク。行き先が狭い次元（`path` の深くなっている次元）と `child` の集合を合わせたもの。
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
    pub(super) fn count(&self) -> usize {
        match self {
            Node::Empty => 0,
            Node::Leaf(_) => 1,
            Node::Branch(branch) => branch.count,
            Node::Skip(skip) => skip.child.count(),
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
    pub(super) fn head_dimension(&self, this: &FlexId) -> Option<Dimension> {
        match self {
            Node::Empty | Node::Leaf(_) => None,
            Node::Branch(branch) => Some(branch.dimension),
            Node::Skip(skip) => this.coarsest_dimension_in(skip.split_dimensions),
        }
    }

    /// 領域 `this` のこの Node の子とその領域を、下側から順に返す。Leaf や空なら何も返さない。
    pub(super) fn children(
        &self,
        this: FlexId,
    ) -> impl DoubleEndedIterator<Item = (&Self, FlexId)> {
        let children = match self {
            Node::Empty | Node::Leaf(_) => [None, None],
            Node::Branch(branch) => {
                let half = |side| this.split_on(branch.dimension, side).unwrap();
                [
                    Some((&branch.lower, half(Side::Lower))),
                    Some((&branch.upper, half(Side::Upper))),
                ]
            }
            Node::Skip(skip) => [
                Some((&skip.child, skip.path.to_absolute(&this).unwrap())),
                None,
            ],
        };
        children.into_iter().flatten()
    }

    /// 同じ Branch・Skip の Arc を指しているなら true。値の型が違う Node とも比べられる。
    pub(super) fn ptr_eq<W, T>(&self, other: &Node<W, T>) -> bool {
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

impl<V, S: Summary<V>> Branch<V, S> {
    fn new(dimension: Dimension, lower: Node<V, S>, upper: Node<V, S>) -> Self {
        let (Some(lower_summary), Some(upper_summary)) = (lower.summary(), upper.summary()) else {
            unreachable!("Branch の子は空にならない")
        };
        Branch {
            dimension,
            split_dimensions: dimension.bit() | lower.split_dimensions() | upper.split_dimensions(),
            height: 1 + lower.height().max(upper.height()),
            count: lower.count() + upper.count(),
            summary: lower_summary.merge(&upper_summary),
            lower,
            upper,
        }
    }
}

impl<V, S> Skip<V, S> {
    fn new(path: RelativeFlexId, child: Node<V, S>) -> Self {
        Skip {
            split_dimensions: path.deeper_dimensions() | child.split_dimensions(),
            path,
            child,
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
    /// `join` が Branch を作るなら true。
    fn forms_branch(dimension: Dimension, lower: &Self, upper: &Self) -> bool {
        // Node は領域との相対位置で意味を持つので、子孫が `dimension` で割っている Node を
        // 広い領域へ持ち上げると意味が変わる。左右が同じでも、そのときは割ったままにする。
        let same = lower.split_dimensions() & dimension.bit() == 0
            && (lower.ptr_eq(upper) || lower == upper);
        !lower.is_empty() && !upper.is_empty() && !same
    }

    /// 領域 `this` を `dimension` で割った左右 `lower`,`upper` を、カノニカル形の1つの[Node]に合わせる。
    fn join(this: &FlexId, dimension: Dimension, lower: Self, upper: Self) -> Self {
        // Branch はここと `merge` でしか作らない。これでどの操作の結果もカノニカル形になる。
        if Node::forms_branch(dimension, &lower, &upper) {
            return Node::Branch(Arc::new(Branch::new(dimension, lower, upper)));
        }
        // カノニカル形では Branch の子は空にならない
        if lower.is_empty() {
            let upper_id = this.split_on(dimension, Side::Upper).unwrap();
            return Node::rebase(upper, &upper_id, this);
        }
        if upper.is_empty() {
            let lower_id = this.split_on(dimension, Side::Lower).unwrap();
            return Node::rebase(lower, &lower_id, this);
        }
        lower
    }

    /// 領域 `from` から見た `node` を、領域 `to` から見た Node にする。`node` の中身は `to` に収まっていること。
    ///
    /// 他と共有していない Skip は、確保し直さずに行き先だけを付け替える。
    pub(super) fn rebase(node: Self, from: &FlexId, to: &FlexId) -> Self {
        match node {
            _ if from == to => node,
            Node::Empty => Node::Empty,
            // Skip の先へさらに Skip しないよう、Skip の行き先から直接張り直す
            Node::Skip(mut skip) => {
                let region = skip.path.to_absolute(from).unwrap();
                if region == *to {
                    return Arc::unwrap_or_clone(skip).child;
                }
                let skip_mut = Arc::make_mut(&mut skip);
                let child = mem::take(&mut skip_mut.child);
                *skip_mut = Skip::new(region.relative_to(to).unwrap(), child);
                Node::Skip(skip)
            }
            node => Node::Skip(Arc::new(Skip::new(from.relative_to(to).unwrap(), node))),
        }
    }

    /// 領域 `this` のこの Node を `dimension` で割った `[下, 上]`（`join` の逆）。
    ///
    /// `dimension` で割っていなければ、両側にこの Node を返す。
    fn split(self, this: &FlexId, dimension: Dimension) -> [Self; 2] {
        if let Node::Skip(skip) = &self
            && skip.path.depth_on(dimension) > 0
        {
            let region = skip.path.to_absolute(this).unwrap();
            let next = this.split_toward(dimension, &region).unwrap();
            let rest = Node::rebase(self, this, &next);
            return if next == this.split_on(dimension, Side::Upper).unwrap() {
                [Node::Empty, rest]
            } else {
                [rest, Node::Empty]
            };
        }
        // その次元では中身が変わらないので、下も上も同じ中身になる
        [self.clone(), self]
    }

    /// 領域 `this` の `a` に `b` を `rule` に従って重ね合わせる。
    ///
    /// `rule` は、`a` と `b` から結果がすぐ決まるなら `a` を書き換えて true を返す。決まらなければ両方を割って子へ降りる。
    /// 他と共有していない Branch・Skip は作り直さずに書き換え、共有しているものは複製してから書き換える。
    pub(super) fn merge<W, T>(
        this: &FlexId,
        a: &mut Self,
        b: View<'_, W, T>,
        rule: &impl Fn(&FlexId, &mut Self, View<'_, W, T>) -> bool,
    ) {
        if rule(this, a, b) {
            return;
        }
        // Skip どうしは、行き先が分かれるところまで1段ずつ割らずに一気に降りる
        if let Node::Skip(skip) = a
            && let Some((region_b, child_b)) = b.as_skip(this)
        {
            let region_a = skip.path.to_absolute(this).unwrap();
            let at = common_descent(this, (&region_a, &skip.child), (&region_b, child_b));
            let b = View::skip(&at, region_b, child_b);
            if at == region_a {
                // 変わるのは行き先の中だけなので、Skip を作り直さずに先の Node を書き換える
                let skip = Arc::make_mut(skip);
                Node::merge(&at, &mut skip.child, b, rule);
                match mem::take(&mut skip.child) {
                    child @ (Node::Leaf(_) | Node::Branch(_)) => {
                        *skip = Skip::new(skip.path, child)
                    }
                    child => *a = Node::rebase(child, &at, this),
                }
                return;
            }
            if at != *this {
                let mut inner = Node::rebase(mem::take(a), this, &at);
                Node::merge(&at, &mut inner, b, rule);
                *a = Node::rebase(inner, &at, this);
                return;
            }
        }
        // 結果もカノニカル形（粗い次元から割る）になるよう、両方の最初の次元のうち粗い方で割る
        let heads = a.head_dimension(this).map_or(0, |d| d.bit())
            | b.head_dimension(this).map_or(0, |d| d.bit());
        let dimension = this
            .coarsest_dimension_in(heads)
            .expect("Leaf どうしは rule が答えを決める");
        let lower_id = this.split_on(dimension, Side::Lower).unwrap();
        let upper_id = this.split_on(dimension, Side::Upper).unwrap();
        let [b_lower, b_upper] = b.split(this, dimension);

        if let Node::Branch(branch) = a
            && branch.dimension == dimension
        {
            let branch = Arc::make_mut(branch);
            Node::merge(&lower_id, &mut branch.lower, b_lower, rule);
            Node::merge(&upper_id, &mut branch.upper, b_upper, rule);
            let lower = mem::take(&mut branch.lower);
            let upper = mem::take(&mut branch.upper);
            if Node::forms_branch(dimension, &lower, &upper) {
                *branch = Branch::new(dimension, lower, upper);
            } else {
                *a = Node::join(this, dimension, lower, upper);
            }
            return;
        }
        let [mut lower, mut upper] = mem::take(a).split(this, dimension);
        Node::merge(&lower_id, &mut lower, b_lower, rule);
        Node::merge(&upper_id, &mut upper, b_upper, rule);
        *a = Node::join(this, dimension, lower, upper);
    }

    /// 和。`a` に値がある場所は `a`、無い場所は `b`。
    pub(super) fn union_rule(this: &FlexId, a: &mut Self, b: View<'_, V, S>) -> bool {
        if a.is_empty() {
            *a = b.to_node(this);
            return true;
        }
        matches!(a, Node::Leaf(_)) || b.is_empty() || b.ptr_eq(a)
    }

    /// 積。`b` に値がある場所だけ `a` を残す。
    pub(super) fn intersection_rule<W, T>(_: &FlexId, a: &mut Self, b: View<'_, W, T>) -> bool {
        if b.is_empty() {
            *a = Node::Empty;
            return true;
        }
        a.is_empty() || b.leaf().is_some() || b.ptr_eq(a)
    }

    /// 差。`b` に値がある場所の `a` を消す。
    pub(super) fn difference_rule<W, T>(_: &FlexId, a: &mut Self, b: View<'_, W, T>) -> bool {
        if b.leaf().is_some() || b.ptr_eq(a) {
            *a = Node::Empty;
            return true;
        }
        a.is_empty() || b.is_empty()
    }

    /// 領域 `this` のこの Node から、条件を満たす Leaf だけを残した Node を返す。何も捨てなければ [`None`]。
    ///
    /// Branch は領域と Summary を `classify` で判断し、子孫をまとめて残すか捨てられるならそれ以上降りない。
    /// Leaf は領域と値を `keep` で判断する。
    pub(super) fn filter(
        &self,
        this: &FlexId,
        classify: &impl Fn(&FlexId, &S) -> Decision,
        keep: &impl Fn(&FlexId, &V) -> bool,
    ) -> Option<Self> {
        match self {
            Node::Empty => None,
            Node::Leaf(value) => (!keep(this, value)).then_some(Node::Empty),
            Node::Branch(branch) => match classify(this, &branch.summary) {
                Decision::KeepAll => None,
                Decision::DropAll => Some(Node::Empty),
                Decision::Descend => {
                    let dimension = branch.dimension;
                    let lower_id = this.split_on(dimension, Side::Lower).unwrap();
                    let upper_id = this.split_on(dimension, Side::Upper).unwrap();
                    let lower = branch.lower.filter(&lower_id, classify, keep);
                    let upper = branch.upper.filter(&upper_id, classify, keep);
                    if lower.is_none() && upper.is_none() {
                        return None;
                    }
                    let lower = lower.unwrap_or_else(|| branch.lower.clone());
                    let upper = upper.unwrap_or_else(|| branch.upper.clone());
                    Some(Node::join(this, dimension, lower, upper))
                }
            },
            Node::Skip(skip) => {
                let region = skip.path.to_absolute(this).unwrap();
                let child = skip.child.filter(&region, classify, keep)?;
                Some(Node::rebase(child, &region, this))
            }
        }
    }
}

/// 行き先 `region`・その先の Node がそれぞれ `a`・`b` の 2 つの Skip が、領域 `this` から
/// 途中で割らずに一緒に降りられる一番狭い領域。
fn common_descent<V, S, W, T>(
    this: &FlexId,
    (region_a, child_a): (&FlexId, &Node<V, S>),
    (region_b, child_b): (&FlexId, &Node<W, T>),
) -> FlexId {
    // Skip の行き先が `region`、その先で割っている次元が `child_split` のとき、領域 `at` で最初に割る次元
    let head = |at: &FlexId, region: &FlexId, child_split: u8| {
        let finer = region.finer_dimensions_than(at);
        (finer != 0).then(|| at.coarsest_dimension_in(finer | child_split))?
    };
    let (split_a, split_b) = (child_a.split_dimensions(), child_b.split_dimensions());
    let mut at = *this;
    while let (Some(da), Some(db)) = (head(&at, region_a, split_a), head(&at, region_b, split_b))
        && da == db
    {
        let next = at.split_toward(da, region_a).unwrap();
        if !next.contains(region_b) {
            break;
        }
        at = next;
    }
    at
}
