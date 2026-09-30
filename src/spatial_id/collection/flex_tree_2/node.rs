use alloc::borrow::Cow;
use alloc::sync::Arc;
use alloc::vec::Vec;

use super::summary::{MinMax, Summary};
use crate::{
    FlexId, Side,
    spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId},
};

/// Node は自分の領域を持たず、親から渡される領域 `this`との相対的な位置で意味を持つ。辿る関数が`this`を持つことで様々な操作を行う。
#[derive(Debug, PartialEq)]
pub(crate) enum Node<V, S> {
    Leaf(Option<V>),
    Branch {
        dimension: Dimension,
        /// 自身と子孫が割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
        split_dimensions: u8,
        /// 自身と子孫で Branch が縦に何段続くかの最大。Node を辿るスタックの大きさに使う。
        height: u8,
        /// 子孫に含まれる、値を持つ[FlexId]の数。
        leaf_count: usize,
        summary: S,
        lower: Arc<Node<V, S>>,
        upper: Arc<Node<V, S>>,
    },
    Skip {
        path: RelativeFlexId,
        /// 自身と子孫が割っている次元のビットマスク。行き先が狭い次元（`path` の深くなっている次元）と `child` の集合を合わせたもの。
        split_dimensions: u8,
        child: Arc<Node<V, S>>,
    },
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
    /// 空の[Node]を作成する。
    pub(super) fn empty() -> Arc<Self> {
        Arc::new(Node::Leaf(None))
    }

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Node::Leaf(None))
    }

    /// 子孫に含まれる、値を持つ[FlexId]の数。
    pub(super) fn leaf_count(&self) -> usize {
        match self {
            Node::Leaf(None) => 0,
            Node::Leaf(Some(_)) => 1,
            Node::Branch { leaf_count, .. } => *leaf_count,
            Node::Skip { child, .. } => child.leaf_count(),
        }
    }

    /// 自身と子孫で Branch が縦に何段続くかの最大。Leaf なら 0。
    pub(super) fn height(&self) -> u8 {
        match self {
            Node::Leaf(_) => 0,
            Node::Branch { height, .. } => *height,
            Node::Skip { child, .. } => child.height(),
        }
    }

    /// 自身と子孫が割っている次元の集合のビットマスク。ビットマスクは[`Dimension::bit`]のOR。
    pub(super) fn split_dimensions(&self) -> u8 {
        match self {
            Node::Leaf(_) => 0,
            Node::Branch {
                split_dimensions, ..
            }
            | Node::Skip {
                split_dimensions, ..
            } => *split_dimensions,
        }
    }

    /// この Node を「最初にどの次元で割るか」。Leaf なら [`None`]。
    fn head_dimension(&self, this: &FlexId) -> Option<Dimension> {
        match self {
            Node::Leaf(_) => None,
            Node::Branch { dimension, .. } => Some(*dimension),
            Node::Skip {
                split_dimensions, ..
            } => this.coarsest_dimension_in(*split_dimensions),
        }
    }

    /// 領域 `this` のこの Node の子を、下側から取り出されるように `stack` へ積む。Leaf なら何もしない。
    pub(super) fn push_children<'a>(&'a self, this: FlexId, stack: &mut Vec<(&'a Self, FlexId)>) {
        match self {
            Node::Leaf(_) => {}
            Node::Branch {
                dimension,
                lower,
                upper,
                ..
            } => {
                stack.push((upper, this.split_on(*dimension, Side::Upper).unwrap()));
                stack.push((lower, this.split_on(*dimension, Side::Lower).unwrap()));
            }
            Node::Skip { path, child, .. } => {
                stack.push((child, path.to_absolute(&this).unwrap()));
            }
        }
    }

    /// 自身が `dimension` で割った Branch で、子が `lower`・`upper` そのもの（同じ Arc）なら true。
    fn has_children(&self, dimension: Dimension, lower: &Arc<Self>, upper: &Arc<Self>) -> bool {
        matches!(self, Node::Branch { dimension: d, lower: l, upper: u, .. }
            if *d == dimension && Arc::ptr_eq(l, lower) && Arc::ptr_eq(u, upper))
    }
}

impl<V, S: AsRef<MinMax<V>>> Node<V, S> {
    /// この Node と子孫の値の範囲 `[min, max]` を返す。空なら [`None`]。
    pub(super) fn value_range(&self) -> Option<(&V, &V)> {
        match self {
            Node::Leaf(None) => None,
            Node::Leaf(Some(v)) => Some((v, v)),
            Node::Branch { summary, .. } => {
                let range = summary.as_ref();
                Some((range.min(), range.max()))
            }
            Node::Skip { child, .. } => child.value_range(),
        }
    }
}

impl<V: PartialEq, S: Summary<V>> Node<V, S> {
    /// この Node と子孫の値の Summary。空なら [`None`]。
    pub(super) fn summary(&self) -> Option<Cow<'_, S>> {
        match self {
            Node::Leaf(None) => None,
            Node::Leaf(Some(value)) => Some(Cow::Owned(S::new(value))),
            Node::Branch { summary, .. } => Some(Cow::Borrowed(summary)),
            Node::Skip { child, .. } => child.summary(),
        }
    }

    /// 領域 `this` を `dimension` で割った左右 `lower`,`upper` を、カノニカル形の1つの[Node]に合わせる。
    fn join(this: &FlexId, dimension: Dimension, lower: Arc<Self>, upper: Arc<Self>) -> Arc<Self> {
        // Branch はここでしか作らない。これでどの操作の結果もカノニカル形になる。
        // Node は領域との相対位置で意味を持つので、子孫が `dimension` で割っている Node を
        // 広い `this` へ持ち上げると意味が変わる。左右が同じでも、そのときは割ったままにする。
        if lower.split_dimensions() & dimension.bit() == 0
            && (Arc::ptr_eq(&lower, &upper) || lower == upper)
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
        Arc::new(Node::Branch {
            dimension,
            split_dimensions: dimension.bit() | lower.split_dimensions() | upper.split_dimensions(),
            height: 1 + lower.height().max(upper.height()),
            leaf_count: lower.leaf_count() + upper.leaf_count(),
            summary,
            lower,
            upper,
        })
    }

    /// 領域 `this` のうち、内側の `target` だけに `child` があり、外側は空の[Node]を作る。
    pub(super) fn skip(this: &FlexId, target: &FlexId, child: Arc<Self>) -> Arc<Self> {
        if target == this {
            return child;
        }
        let (target_id, child) = match &*child {
            // 空はどこに置いても空
            Node::Leaf(None) => return child,
            // Skip の先へさらに Skip しないよう、Skip の行き先から直接張り直す
            Node::Skip {
                path, child: inner, ..
            } => (path.to_absolute(target).unwrap(), inner.clone()),
            _ => (*target, child),
        };
        let path = target_id.relative_to(this).unwrap();
        let split_dimensions = path.deeper_dimensions() | child.split_dimensions();
        Arc::new(Node::Skip {
            path,
            split_dimensions,
            child,
        })
    }

    /// 2 つの Node `a`・`b` を、領域 `this` の上で `merge_rule` に従って重ね合わせる。
    ///
    /// `empty` は、結果の空の場所に使う空の Leaf（`a` 側の型と `b` 側の型）。
    pub(super) fn merge<W: PartialEq, T: Summary<W>>(
        this: &FlexId,
        a: &Arc<Self>,
        b: &Arc<Node<W, T>>,
        merge_rule: &impl Fn(&Arc<Self>, &Arc<Node<W, T>>, &Arc<Self>) -> Option<Arc<Self>>,
        empty: &(Arc<Self>, Arc<Node<W, T>>),
    ) -> Arc<Self> {
        // 空の Leaf を毎回確保しないよう、呼び出し側が操作ごとに1組だけ作った `empty` を使い回す
        if let Some(result) =
            merge_rule(a, b, &empty.0).or_else(|| Node::merge_skips(this, a, b, merge_rule, empty))
        {
            return result;
        }

        // 結果もカノニカル形（粗い次元から割る）になるよう、両方の最初の次元のうち粗い方で割る
        let heads = a.head_dimension(this).map_or(0, |d| d.bit())
            | b.head_dimension(this).map_or(0, |d| d.bit());
        let dimension = this
            .coarsest_dimension_in(heads)
            .expect("Leaf どうしは merge_rule が答えを決める");

        let [a_lower, a_upper] = Node::split(this, a, dimension, &empty.0);
        let [b_lower, b_upper] = Node::split(this, b, dimension, &empty.1);
        let lower_id = this.split_on(dimension, Side::Lower).unwrap();
        let upper_id = this.split_on(dimension, Side::Upper).unwrap();
        let lower = Node::merge(&lower_id, &a_lower, &b_lower, merge_rule, empty);
        let upper = Node::merge(&upper_id, &a_upper, &b_upper, merge_rule, empty);

        if a.has_children(dimension, &lower, &upper) {
            return a.clone();
        }
        Node::join(this, dimension, lower, upper)
    }

    /// `a`・`b` がどちらも Skip で、2 つの行き先を共に含む `this` より狭い領域があれば、
    /// その領域で重ね合わせた結果を返す。無ければ [`None`]。
    fn merge_skips<W: PartialEq, T: Summary<W>>(
        this: &FlexId,
        a: &Arc<Self>,
        b: &Arc<Node<W, T>>,
        rule: &impl Fn(&Arc<Self>, &Arc<Node<W, T>>, &Arc<Self>) -> Option<Arc<Self>>,
        empty: &(Arc<Self>, Arc<Node<W, T>>),
    ) -> Option<Arc<Self>> {
        let (
            Node::Skip {
                path: path_a,
                child: child_a,
                ..
            },
            Node::Skip {
                path: path_b,
                child: child_b,
                ..
            },
        ) = (&**a, &**b)
        else {
            return None;
        };
        // 行き先が分かれるまでは途中の Node を作らずに降り、分かれる地点で1回だけ重ね合わせる
        let region_a = path_a.to_absolute(this).unwrap();
        let region_b = path_b.to_absolute(this).unwrap();
        let (split_a, split_b) = (child_a.split_dimensions(), child_b.split_dimensions());

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

        let a = Node::skip(&at, &region_a, child_a.clone());
        let b = Node::skip(&at, &region_b, child_b.clone());
        Some(Node::skip(this, &at, Node::merge(&at, &a, &b, rule, empty)))
    }

    /// 領域 `this` の `node` を `dimension` で割った `[下, 上]`（`join` の逆）。空の側は `empty` を使う。
    ///
    /// `node` が `dimension` で割っていなければ、両側に `node` を返す。
    fn split(
        this: &FlexId,
        node: &Arc<Self>,
        dimension: Dimension,
        empty: &Arc<Self>,
    ) -> [Arc<Self>; 2] {
        match &**node {
            Node::Branch {
                dimension: d,
                lower,
                upper,
                ..
            } if *d == dimension => [lower.clone(), upper.clone()],
            Node::Skip { path, child, .. } if path.depth_on(dimension) > 0 => {
                let region = path.to_absolute(this).unwrap();
                let next = this.split_toward(dimension, &region).unwrap();
                let rest = Node::skip(&next, &region, child.clone());
                if next == this.split_on(dimension, Side::Upper).unwrap() {
                    [empty.clone(), rest]
                } else {
                    [rest, empty.clone()]
                }
            }
            // その次元では中身が変わらないので、下も上も同じ中身になる
            _ => [node.clone(), node.clone()],
        }
    }

    /// 和。`a` に値がある場所は `a`、無い場所は `b`。
    pub(super) fn union_rule(
        a: &Arc<Self>,
        b: &Arc<Self>,
        _empty: &Arc<Self>,
    ) -> Option<Arc<Self>> {
        match (&**a, &**b) {
            _ if Arc::ptr_eq(a, b) => Some(a.clone()),
            (Node::Leaf(Some(_)), _) | (_, Node::Leaf(None)) => Some(a.clone()),
            (Node::Leaf(None), _) => Some(b.clone()),
            _ => None,
        }
    }

    /// 積。`b` に値がある場所だけ `a` を残す。
    pub(super) fn intersection_rule<W: PartialEq, T: Summary<W>>(
        a: &Arc<Self>,
        b: &Arc<Node<W, T>>,
        empty: &Arc<Self>,
    ) -> Option<Arc<Self>> {
        if Arc::as_ptr(a) as *const () == Arc::as_ptr(b) as *const () {
            return Some(a.clone());
        }
        match (&**a, &**b) {
            (Node::Leaf(None), _) | (_, Node::Leaf(Some(_))) => Some(a.clone()),
            (_, Node::Leaf(None)) => Some(empty.clone()),
            _ => None,
        }
    }

    /// 差。`b` に値がある場所の `a` を消す。
    pub(super) fn difference_rule<W: PartialEq, T: Summary<W>>(
        a: &Arc<Self>,
        b: &Arc<Node<W, T>>,
        empty: &Arc<Self>,
    ) -> Option<Arc<Self>> {
        if Arc::as_ptr(a) as *const () == Arc::as_ptr(b) as *const () {
            return Some(empty.clone());
        }
        match (&**a, &**b) {
            (Node::Leaf(None), _) | (_, Node::Leaf(None)) => Some(a.clone()),
            (_, Node::Leaf(Some(_))) => Some(empty.clone()),
            _ => None,
        }
    }

    /// 領域 `this` の `node` から、条件を満たす Leaf だけを残す。捨てた場所には `empty` を使う。
    ///
    /// Branch は領域と Summary を `classify` で判断し、子孫をまとめて残すか捨てられるならそれ以上降りない。
    /// Leaf は領域と値を `keep` で判断する。変化の無い Node は元の [Arc] をそのまま使う。
    pub(super) fn filter(
        this: &FlexId,
        node: &Arc<Self>,
        classify: &impl Fn(&FlexId, &S) -> Decision,
        keep: &impl Fn(&FlexId, &V) -> bool,
        empty: &Arc<Self>,
    ) -> Arc<Self> {
        match &**node {
            Node::Leaf(None) => node.clone(),
            Node::Leaf(Some(value)) if keep(this, value) => node.clone(),
            Node::Leaf(Some(_)) => empty.clone(),
            Node::Branch {
                dimension,
                summary,
                lower,
                upper,
                ..
            } => match classify(this, summary) {
                Decision::KeepAll => node.clone(),
                Decision::DropAll => empty.clone(),
                Decision::Descend => {
                    let lower_id = this.split_on(*dimension, Side::Lower).unwrap();
                    let upper_id = this.split_on(*dimension, Side::Upper).unwrap();
                    let lower = Node::filter(&lower_id, lower, classify, keep, empty);
                    let upper = Node::filter(&upper_id, upper, classify, keep, empty);
                    if node.has_children(*dimension, &lower, &upper) {
                        return node.clone();
                    }
                    Node::join(this, *dimension, lower, upper)
                }
            },
            Node::Skip { path, child, .. } => {
                let region = path.to_absolute(this).unwrap();
                let new_child = Node::filter(&region, child, classify, keep, empty);
                if Arc::ptr_eq(child, &new_child) {
                    return node.clone();
                }
                Node::skip(this, &region, new_child)
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
