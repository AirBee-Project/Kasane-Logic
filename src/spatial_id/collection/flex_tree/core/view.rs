use super::node::Node;
use super::summary::Summary;
use crate::{FlexId, Side, spatial_id::dimension::Dimension};

/// 領域 `this` の上の Node を借りて見たもの。
pub(super) enum View<'a, V, S> {
    Empty,
    Node(&'a Node<V, S>),
    /// 領域のうち `region` だけに `child` があり、外側は空。`child` は空でも Skip でもない。
    Skip {
        region: FlexId,
        child: &'a Node<V, S>,
    },
}

// derive では `V: Copy` まで要求されるが、参照の複製に値の複製は要らない
impl<V, S> Clone for View<'_, V, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V, S> Copy for View<'_, V, S> {}

impl<'a, V, S> From<&'a Node<V, S>> for View<'a, V, S> {
    fn from(node: &'a Node<V, S>) -> Self {
        match node {
            Node::Empty => View::Empty,
            node => View::Node(node),
        }
    }
}

impl<'a, V, S> View<'a, V, S> {
    /// 領域 `this` のうち `region` だけに `child` がある View。
    pub(super) fn skip(this: &FlexId, region: FlexId, child: &'a Node<V, S>) -> Self {
        match child {
            _ if region == *this => View::from(child),
            Node::Empty => View::Empty,
            Node::Skip(skip) => View::Skip {
                region: skip.path.to_absolute(&region).unwrap(),
                child: &skip.child,
            },
            _ => View::Skip { region, child },
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, View::Empty)
    }

    /// Leaf ならその値。
    pub(super) fn leaf(&self) -> Option<&'a V> {
        match self {
            View::Node(Node::Leaf(value)) => Some(value),
            _ => None,
        }
    }

    /// `node` と同じ Branch・Skip の Arc を見ているなら true。値の型が違う Node とも比べられる。
    pub(super) fn ptr_eq<W, T>(&self, node: &Node<W, T>) -> bool {
        matches!(self, View::Node(this) if this.ptr_eq(node))
    }

    /// Skip なら、行き先の領域とその先の Node。
    pub(super) fn as_skip(&self, this: &FlexId) -> Option<(FlexId, &'a Node<V, S>)> {
        match *self {
            View::Node(Node::Skip(skip)) => {
                Some((skip.path.to_absolute(this).unwrap(), &skip.child))
            }
            View::Skip { region, child } => Some((region, child)),
            _ => None,
        }
    }

    /// 最初にどの次元で割るか。Leaf や空なら [`None`]。
    pub(super) fn head_dimension(&self, this: &FlexId) -> Option<Dimension> {
        match self {
            View::Empty => None,
            View::Node(node) => node.head_dimension(this),
            View::Skip { region, child } => this.coarsest_dimension_in(
                region.finer_dimensions_than(this) | child.split_dimensions(),
            ),
        }
    }

    /// `dimension` で割った `[下, 上]`。`dimension` で割っていなければ、両側に自身を返す。
    ///
    /// `halves` は `this` を `dimension` で割った `[下, 上]` の領域。
    pub(super) fn split(
        &self,
        this: &FlexId,
        dimension: Dimension,
        halves: &[FlexId; 2],
    ) -> [Self; 2] {
        let (region, child) = match *self {
            View::Node(Node::Branch(branch)) if branch.dimension == dimension => {
                return [View::from(&branch.lower), View::from(&branch.upper)];
            }
            // 相対位置で持つ Skip は、そのまま半分の領域から見ても同じ中身になる
            View::Node(Node::Skip(skip)) if skip.path.depth_on(dimension) == 0 => {
                return [*self, *self];
            }
            _ => match self.as_skip(this) {
                Some(skip) => skip,
                None => return [*self, *self],
            },
        };
        if region.finer_dimensions_than(this) & dimension.bit() != 0 {
            let side = this.side_toward(dimension, &region);
            let rest = View::skip(&halves[side as usize], region, child);
            return match side {
                Side::Lower => [rest, View::Empty],
                Side::Upper => [View::Empty, rest],
            };
        }
        // その次元では中身が変わらないので、下も上も同じ中身になる。
        // 行き先は絶対位置なので、それぞれの半分に切り詰める
        halves.map(|half| View::skip(&half, region.intersection(&half).unwrap(), child))
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> View<'_, V, S> {
    /// 領域 `this` の Node として取り出す。
    pub(super) fn to_node(self, this: &FlexId) -> Node<V, S> {
        match self {
            View::Empty => Node::Empty,
            View::Node(node) => node.clone(),
            View::Skip { region, child } => Node::rebase(child.clone(), &region, this),
        }
    }
}
