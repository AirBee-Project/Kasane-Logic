use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::{
    FlexId, Side,
    spatial_id::{dimension::Dimension, relative_flex_id::RelativeFlexId},
};

/// ノードは自分の領域を持たず、親から渡される領域 `this`との相対的な位置で意味を持つ。辿る関数が`this`を持つことで様々な操作を行う。
#[derive(Debug, PartialEq)]
pub(crate) enum Node<V> {
    Leaf(Option<V>),
    Branch {
        dimension: Dimension,
        /// 配下で割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
        split_dimensions: u8,
        lower: Arc<Node<V>>,
        upper: Arc<Node<V>>,
    },
    Skip {
        path: RelativeFlexId,
        /// 配下で割っている次元のビットマスク。自身の `dimension` とlowerとupperの集合を合わせたもの。
        split_dimensions: u8,
        child: Arc<Node<V>>,
    },
}

impl<V: Clone + PartialEq> Node<V> {
    /// 空の[Node]を作成する。
    pub(super) fn empty() -> Arc<Self> {
        Arc::new(Node::Leaf(None))
    }

    /// 領域 `this` のうち、`target` だけに `value`がある[Node]を作る。
    pub(super) fn only_at(this: &FlexId, target: &FlexId, value: V) -> Arc<Self> {
        Node::skip(this, target, Arc::new(Node::Leaf(Some(value))))
    }

    /// 領域 `this` を `dimension` で割った左右 `lower`,`upper` を、1つの[Node]に合わせる。木を組み立てる経路はここだけで、結果は必ずカノニカルになる。
    fn join(this: &FlexId, dimension: Dimension, lower: Arc<Self>, upper: Arc<Self>) -> Arc<Self> {
        // 両側が同じで、その次元で値が変わらないなら、割らずに片方を返す（同じ値の葉どうしもここ）
        if lower.split_dimensions() & dimension.bit() == 0
            && (Arc::ptr_eq(&lower, &upper) || lower == upper)
        {
            return lower;
        }
        // 片側が空なら、残りを Skip で包む
        if lower.is_empty() {
            let upper_id = this.split_on(dimension, Side::Upper).unwrap();
            return Node::skip(this, &upper_id, upper);
        }
        if upper.is_empty() {
            let lower_id = this.split_on(dimension, Side::Lower).unwrap();
            return Node::skip(this, &lower_id, lower);
        }
        Arc::new(Node::Branch {
            dimension,
            split_dimensions: dimension.bit() | lower.split_dimensions() | upper.split_dimensions(),
            lower,
            upper,
        })
    }

    /// 領域 `this` のうち、内側の `target` だけに `child` があり、外側は空の[Node]を作る。
    fn skip(this: &FlexId, target: &FlexId, child: Arc<Self>) -> Arc<Self> {
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

    /// 2 つの木 `a`・`b` を、領域 `this` の上で同時にたどって重ね合わせる。
    pub(super) fn merge<W: Clone + PartialEq>(
        this: &FlexId,
        a: &Arc<Self>,
        b: &Arc<Node<W>>,
        merge_rule: &impl Fn(&Arc<Self>, &Arc<Node<W>>) -> Option<Arc<Self>>,
    ) -> Arc<Self> {
        if let Some(result) = merge_rule(a, b).or_else(|| Node::merge_skips(this, a, b, merge_rule))
        {
            return result;
        }

        // 両方が最初に割る次元のうち粗い方で割る
        let heads = a.head_dimension(this).map_or(0, |d| d.bit())
            | b.head_dimension(this).map_or(0, |d| d.bit());
        let dimension = this
            .coarsest_dimension_in(heads)
            .expect("葉どうしは merge_rule が答えを決める");

        let [a_lower, a_upper] = Node::split(this, a, dimension);
        let [b_lower, b_upper] = Node::split(this, b, dimension);
        let lower_id = this.split_on(dimension, Side::Lower).unwrap();
        let upper_id = this.split_on(dimension, Side::Upper).unwrap();
        let lower = Node::merge(&lower_id, &a_lower, &b_lower, merge_rule);
        let upper = Node::merge(&upper_id, &a_upper, &b_upper, merge_rule);

        // 変化が無ければ a をそのまま使う
        if a.has_children(dimension, &lower, &upper) {
            return a.clone();
        }
        Node::join(this, dimension, lower, upper)
    }

    /// 自身が `dimension` で割った Branch で、子が `lower`・`upper` そのもの（同じ Arc）なら true。
    fn has_children(&self, dimension: Dimension, lower: &Arc<Self>, upper: &Arc<Self>) -> bool {
        matches!(self, Node::Branch { dimension: d, lower: l, upper: u, .. }
            if *d == dimension && Arc::ptr_eq(l, lower) && Arc::ptr_eq(u, upper))
    }

    /// 両方が Skip なら、2 つの行き先が同じ側にある間はノードを作らずに降り、
    /// 分かれる地点で 1 回だけ重ね合わせる。どちらかが Skip でなければ [`None`]。
    fn merge_skips<W: Clone + PartialEq>(
        this: &FlexId,
        a: &Arc<Self>,
        b: &Arc<Node<W>>,
        rule: &impl Fn(&Arc<Self>, &Arc<Node<W>>) -> Option<Arc<Self>>,
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
        Some(Node::skip(this, &at, Node::merge(&at, &a, &b, rule)))
    }

    /// このノードを「最初にどの次元で割るか」。Leaf なら [`None`]。カノニカルな Skip は行き先が狭い次元から割るので、キャッシュ済みの集合から一番粗い次元を選べばよい。
    fn head_dimension(&self, this: &FlexId) -> Option<Dimension> {
        match self {
            Node::Leaf(_) => None,
            Node::Branch { dimension, .. } => Some(*dimension),
            Node::Skip {
                split_dimensions, ..
            } => this.coarsest_dimension_in(*split_dimensions),
        }
    }

    /// 領域 `this` の `node` を `dimension` で割った `[下, 上]`（`join` の逆）。
    ///
    /// `node` が `dimension` で割っていなければ、その次元では中身が変わらないので、
    /// 下も上も `node` と同じ中身になる（両側に `node` を返す）。
    fn split(this: &FlexId, node: &Arc<Self>, dimension: Dimension) -> [Arc<Self>; 2] {
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
                    [Node::empty(), rest]
                } else {
                    [rest, Node::empty()]
                }
            }
            _ => [node.clone(), node.clone()],
        }
    }

    /// 配下で割っている次元の集合のビットマスク。ビットマスクは[`Dimension::bit`]のOR。
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

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Node::Leaf(None))
    }

    /// 領域 `this` のこのノード配下で、値を持つ領域を `out` へ集める。
    pub(super) fn collect<'a>(&'a self, this: &FlexId, out: &mut Vec<(FlexId, &'a V)>) {
        match self {
            Node::Leaf(None) => {}
            Node::Leaf(Some(value)) => out.push((*this, value)),
            Node::Branch {
                dimension,
                lower,
                upper,
                ..
            } => {
                let lower_id = this.split_on(*dimension, Side::Lower).unwrap();
                let upper_id = this.split_on(*dimension, Side::Upper).unwrap();
                lower.collect(&lower_id, out);
                upper.collect(&upper_id, out);
            }
            Node::Skip { path, child, .. } => {
                let region = path.to_absolute(this).unwrap();
                child.collect(&region, out);
            }
        }
    }

    /// 上書き（insert）。`b` に値がある場所は `b`、無い場所は `a`。
    pub(super) fn overwrite_rule(a: &Arc<Self>, b: &Arc<Self>) -> Option<Arc<Self>> {
        match (&**a, &**b) {
            _ if Arc::ptr_eq(a, b) => Some(a.clone()),
            (_, Node::Leaf(None)) => Some(a.clone()),
            (Node::Leaf(None), _) | (_, Node::Leaf(Some(_))) => Some(b.clone()),
            _ => None,
        }
    }

    /// 和。`a` に値がある場所は `a`、無い場所は `b`。
    pub(super) fn union_rule(a: &Arc<Self>, b: &Arc<Self>) -> Option<Arc<Self>> {
        match (&**a, &**b) {
            _ if Arc::ptr_eq(a, b) => Some(a.clone()),
            (Node::Leaf(Some(_)), _) | (_, Node::Leaf(None)) => Some(a.clone()),
            (Node::Leaf(None), _) => Some(b.clone()),
            _ => None,
        }
    }

    /// 積。`b` に値がある場所だけ `a` を残す。
    pub(super) fn intersection_rule<W>(a: &Arc<Self>, b: &Arc<Node<W>>) -> Option<Arc<Self>> {
        match (&**a, &**b) {
            (Node::Leaf(None), _) | (_, Node::Leaf(Some(_))) => Some(a.clone()),
            (_, Node::Leaf(None)) => Some(Node::empty()),
            _ => None,
        }
    }

    /// 差。`b` に値がある場所の `a` を消す。
    pub(super) fn difference_rule<W>(a: &Arc<Self>, b: &Arc<Node<W>>) -> Option<Arc<Self>> {
        match (&**a, &**b) {
            (Node::Leaf(None), _) | (_, Node::Leaf(None)) => Some(a.clone()),
            (_, Node::Leaf(Some(_))) => Some(Node::empty()),
            _ => None,
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
