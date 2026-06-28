use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::{
    Dimension, FlexId, IntoSingleIds, IterFlexIds, RangeId, Side, SingleId, SpatialId,
    spatial_id::collection::flex_tree::core::convert::{LeavesIter, LeavesIterRef},
};
use node::Node;
mod convert;
pub mod node;
pub mod node_ops;
mod overlap;
pub(crate) mod ptr;
use ptr::SharedNode;

/// 拡張空間IDとそれに紐づいたValueを保存するための型
#[derive(Clone, Debug)]
#[cfg_attr(
    feature = "persist",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[cfg_attr(feature = "persist", rkyv(archive_bounds(V: 'static)))]
#[cfg_attr(
    feature = "persist",
    rkyv(serialize_bounds(
        __S: rkyv::ser::Writer + rkyv::ser::Allocator + rkyv::ser::Sharing,
        <__S as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(deserialize_bounds(
        __D: rkyv::de::Pooling,
        <__D as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    ))
)]
#[cfg_attr(
    feature = "persist",
    rkyv(bytecheck(bounds(
        __C: rkyv::validation::ArchiveContext + rkyv::validation::SharedContext,
        <__C as rkyv::rancor::Fallible>::Error: rkyv::rancor::Source,
    )))
)]
pub struct FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    pub(crate) lower_root: SharedNode<Node<V>>,
    pub(crate) upper_root: SharedNode<Node<V>>,
    pub(crate) empty_leaf: SharedNode<Node<V>>,

    /// このツリーが閉じているシャード領域。`None` は全空間。
    /// `Some(region)` のとき、`region` の外側への挿入は無視される。
    pub(crate) shard: Option<FlexId>,
}

impl<V> Default for FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PartialEq for FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    fn eq(&self, other: &Self) -> bool {
        self.lower_root == other.lower_root && self.upper_root == other.upper_root
    }
}

impl<V> Eq for FlexTreeCore<V> where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue
{
}

/// union 結果のシャード領域を決める。両者が同一領域ならそれを保ち、
/// 異なる・無制限が絡む場合は全空間（`None`）とみなす。
fn shard_after_union(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
    match (a, b) {
        (Some(a), Some(b)) if a == b => Some(a.clone()),
        _ => None,
    }
}

/// intersection 結果のシャード領域を、交差した狭い領域へ絞り込む。
fn shard_after_intersection(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
    match (a, b) {
        (Some(a), Some(b)) => a.intersection(b).or_else(|| Some(a.clone())),
        (Some(a), None) => Some(a.clone()),
        (None, Some(b)) => Some(b.clone()),
        (None, None) => None,
    }
}

impl<V> FlexTreeCore<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// 新しい空の[FlexTreeCore]を作成する
    pub fn new() -> Self {
        let empty_leaf = SharedNode::new(Node::Leaf { value: None });
        Self {
            lower_root: empty_leaf.clone(),
            upper_root: empty_leaf.clone(),
            empty_leaf,
            shard: None,
        }
    }

    /// シャード領域 `region` に閉じた空の[FlexTreeCore]を作成する。
    ///
    /// 以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        let mut core = Self::new();
        core.shard = Some(region);
        core
    }

    /// このツリーが閉じているシャード領域を返す。`None` は全空間。
    pub(crate) fn shard(&self) -> Option<&FlexId> {
        self.shard.as_ref()
    }

    /// 2つの [FlexTreeCore] の和集合を計算します。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            lower_root: Node::union(&self.lower_root, &other.lower_root, 0, &self.empty_leaf),
            upper_root: Node::union(&self.upper_root, &other.upper_root, 0, &self.empty_leaf),
            empty_leaf: self.empty_leaf.clone(),
            shard: shard_after_union(&self.shard, &other.shard),
        }
    }

    pub fn intersection(&self, other: &Self) -> Self {
        if let (Some(a), Some(b)) = (&self.shard, &other.shard)
            && a.intersection(b).is_none()
        {
            return Self {
                lower_root: self.empty_leaf.clone(),
                upper_root: self.empty_leaf.clone(),
                empty_leaf: self.empty_leaf.clone(),
                shard: shard_after_intersection(&self.shard, &other.shard),
            };
        }

        Self {
            lower_root: Node::intersection(
                &self.lower_root,
                &other.lower_root,
                0,
                &self.empty_leaf,
            ),
            upper_root: Node::intersection(
                &self.upper_root,
                &other.upper_root,
                0,
                &self.empty_leaf,
            ),
            empty_leaf: self.empty_leaf.clone(),
            shard: shard_after_intersection(&self.shard, &other.shard),
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        if let (Some(a), Some(b)) = (&self.shard, &other.shard)
            && a.intersection(b).is_none()
        {
            return self.clone();
        }

        Self {
            lower_root: Node::difference(&self.lower_root, &other.lower_root, 0, &self.empty_leaf),
            upper_root: Node::difference(&self.upper_root, &other.upper_root, 0, &self.empty_leaf),
            empty_leaf: self.empty_leaf.clone(),
            shard: self.shard.clone(),
        }
    }

    /// ルートノードのポインタが完全に同一か判定します（Result Reuseテスト用）
    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        SharedNode::ptr_eq(&self.lower_root, &other.lower_root)
            && SharedNode::ptr_eq(&self.upper_root, &other.upper_root)
    }

    ///クリアする
    pub fn clear(&mut self) {
        self.lower_root = self.empty_leaf.clone();
        self.upper_root = self.empty_leaf.clone();
    }

    pub fn is_empty(&self) -> bool {
        self.lower_root.leaf_count() == 0 && self.upper_root.leaf_count() == 0
    }

    pub fn count(&self) -> usize {
        self.lower_root.leaf_count() + self.upper_root.leaf_count()
    }

    /// このFlexTreeをシャードのために分割する際に、どこを切り出せば良いのかを判断する関数。木の全体を見て、
    pub(crate) fn balanced_cut(&self) -> Option<FlexId> {
        let lower = self.lower_root.leaf_count();
        let upper = self.upper_root.leaf_count();
        let total = lower + upper;
        if total == 0 {
            return None;
        }

        let half = total as i64 / 2;
        let closeness = |count: usize| (count as i64 - half).unsigned_abs();

        let (heavy_root, heavy_id, light_id, light_count) = if lower >= upper {
            (
                &self.lower_root,
                FlexId::LOWER_MAX,
                FlexId::UPPER_MAX,
                upper,
            )
        } else {
            (
                &self.upper_root,
                FlexId::UPPER_MAX,
                FlexId::LOWER_MAX,
                lower,
            )
        };

        let mut best_region = light_id;
        let mut best_count = light_count;

        let mut node = heavy_root.clone();
        let mut id = heavy_id;
        loop {
            let branch = match &*node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => Some((
                    *level,
                    lower_child.leaf_count(),
                    upper_child.leaf_count(),
                    lower_child.clone(),
                    upper_child.clone(),
                )),
                Node::Leaf { .. } => None,
            };
            let Some((level, l, u, lower_child, upper_child)) = branch else {
                break;
            };

            let axis = Node::<V>::axis(level);
            let lower_id = split_child_id(&id, axis, Side::Lower);
            let upper_id = split_child_id(&id, axis, Side::Upper);

            if closeness(l) < closeness(best_count) {
                best_region = lower_id.clone();
                best_count = l;
            }
            if closeness(u) < closeness(best_count) {
                best_region = upper_id.clone();
                best_count = u;
            }

            // 偏りの原因は重い子の中にある → まだ過半なら重い子へ降りる。
            let (heavy_child, heavy_child_id, heavy_count) = if l >= u {
                (lower_child, lower_id, l)
            } else {
                (upper_child, upper_id, u)
            };
            if heavy_count as i64 > half {
                node = heavy_child;
                id = heavy_child_id;
            } else {
                break;
            }
        }

        Some(best_region)
    }

    /// この[FlexTree]をシャード分割すべきかを判定する。保持する[FlexId]数が `max_flex_id_count` を超えていれば `true`を返す。[FlexId]の個数はキャッシュされているため高速に動作する。
    pub fn should_split_shard(&self, max_flex_id_count: usize) -> bool {
        self.count() > max_flex_id_count
    }

    /// [FlexTree]を互いに素なシャードへ分割する。この時、シャード1つあたりの中身の[FlexId]の個数は`max_flex_id_count` 以下になる。分割の必要がなければ、自分自身を返す。
    pub fn split_shard(&self, max_flex_id_count: usize) -> Vec<Self> {
        let mut result = Vec::new();
        let mut pending = alloc::vec![self.clone()];

        while let Some(piece) = pending.pop() {
            // 閾値以下、または分割不能（1要素以下）ならそのまま確定
            if piece.count() <= max_flex_id_count || piece.count() < 2 {
                result.push(piece);
                continue;
            }

            let Some(region) = piece.balanced_cut() else {
                result.push(piece);
                continue;
            };

            let sub_pieces: Vec<Self> = piece
                .shard_regions(region)
                .into_iter()
                .map(|piece_region| piece.extract_region(piece_region))
                .collect();
            pending.extend(sub_pieces);
        }

        result
    }

    pub(crate) fn shard_regions(&self, region: FlexId) -> Vec<FlexId> {
        let mut regions = alloc::vec![region.clone()];

        // R が属するルートと、反対側ルート。
        let in_lower = region.f_index() < 0;
        let (region_root, region_root_id, other_root, other_root_id) = if in_lower {
            (
                &self.lower_root,
                FlexId::LOWER_MAX,
                &self.upper_root,
                FlexId::UPPER_MAX,
            )
        } else {
            (
                &self.upper_root,
                FlexId::UPPER_MAX,
                &self.lower_root,
                FlexId::LOWER_MAX,
            )
        };

        if other_root.leaf_count() > 0 {
            regions.push(other_root_id);
        }

        let mut node = region_root.clone();
        let mut id = region_root_id;
        while id != region {
            let next = match &*node {
                Node::Branch {
                    level,
                    lower_child,
                    upper_child,
                    ..
                } => {
                    let axis = Node::<V>::axis(*level);
                    let lower_id = split_child_id(&id, axis, Side::Lower);
                    let upper_id = split_child_id(&id, axis, Side::Upper);
                    if lower_id.intersection(&region).is_some() {
                        if upper_child.leaf_count() > 0 {
                            regions.push(upper_id);
                        }
                        (lower_child.clone(), lower_id)
                    } else {
                        if lower_child.leaf_count() > 0 {
                            regions.push(lower_id);
                        }
                        (upper_child.clone(), upper_id)
                    }
                }
                Node::Leaf { .. } => break,
            };
            node = next.0;
            id = next.1;
        }

        regions
    }

    fn extract_region(&self, region: FlexId) -> Self {
        let in_lower = region.f_index() < 0;

        let mut piece = self.clone();
        {
            let (root, root_id) = if in_lower {
                (&mut piece.lower_root, FlexId::LOWER_MAX)
            } else {
                (&mut piece.upper_root, FlexId::UPPER_MAX)
            };
            Self::prune_path(root, root_id, &region, true, &self.empty_leaf);
        }
        if in_lower {
            piece.upper_root = self.empty_leaf.clone();
        } else {
            piece.lower_root = self.empty_leaf.clone();
        }
        piece.shard = Some(region);
        piece
    }

    fn prune_path(
        node: &mut SharedNode<Node<V>>,
        current_id: FlexId,
        region: &FlexId,
        keep: bool,
        empty_leaf: &SharedNode<Node<V>>,
    ) {
        if &current_id == region {
            if !keep {
                *node = empty_leaf.clone();
            }
            return;
        }

        let mut_node = SharedNode::make_mut(node);
        if let Node::Branch {
            level,
            lower_child,
            upper_child,
            leaf_count,
            max_zoom,
        } = mut_node
        {
            let axis = Node::<V>::axis(*level);
            let lower_id = split_child_id(&current_id, axis, Side::Lower);
            let upper_id = split_child_id(&current_id, axis, Side::Upper);

            // region は子のちょうど一方に含まれる。
            if lower_id.intersection(region).is_some() {
                if keep {
                    *upper_child = empty_leaf.clone();
                }
                Self::prune_path(lower_child, lower_id, region, keep, empty_leaf);
            } else {
                if keep {
                    *lower_child = empty_leaf.clone();
                }
                Self::prune_path(upper_child, upper_id, region, keep, empty_leaf);
            }

            *leaf_count = lower_child.leaf_count() + upper_child.leaf_count();
            *max_zoom = Node::<V>::fold_max_zoom(*level, lower_child, upper_child);
        }
    }

    /// この [`FlexTreeCore`] に含まれる要素のうち、最も高いズームレベル値を返します。
    ///
    /// ここでいう解像度は、各 [`FlexId`] の `f/x/y` それぞれのズームレベルの最大値です。
    /// 空の木では [`None`] を返します。
    ///
    /// # 例
    /// ```
    /// # use kasane_logic::{spatial_id::collection::flex_tree::core::FlexTreeCore, RangeId, SingleId};
    /// let mut core = FlexTreeCore::new();
    /// core.insert(RangeId::new(4, [0, 1], [0, 0], [0, 0]).unwrap(), ());
    /// assert_eq!(core.max_zoomlevel(), Some(4));
    /// ```
    pub fn max_zoomlevel(&self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        // 各 Branch がキャッシュした max_zoom を畳み上げるだけなので O(1)。ルートはレベル 0。
        let lower = self.lower_root.max_zoom_at(0);
        let upper = self.upper_root.max_zoom_at(0);
        Some(lower.max(upper))
    }

    /// この集合が値を持つ全セルを包む最小の[RangeId]を返します。
    /// # 例
    /// ```
    /// # use kasane_logic::{spatial_id::collection::flex_tree::core::FlexTreeCore, SingleId};
    /// let mut core = FlexTreeCore::new();
    /// core.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
    /// core.insert(SingleId::new(20, 0, 2, 3).unwrap(), 1);
    ///
    /// let bbox = core.bounding_box().unwrap();
    /// assert_eq!(bbox.z(), 20);
    /// assert_eq!(bbox.f(), [0, 0]);
    /// assert_eq!(bbox.x(), [0, 2]);
    /// assert_eq!(bbox.y(), [0, 3]);
    ///
    /// let empty: FlexTreeCore<i32> = FlexTreeCore::new();
    /// assert!(empty.bounding_box().is_none());
    /// ```
    pub fn bounding_box(&self) -> Option<RangeId> {
        RangeId::bounding_box_of(self.iter().map(|(flex_id, _)| flex_id))
    }

    /// この [`FlexTreeCore`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として書き出します。
    ///
    /// 返される `SingleId` はすべて同じズームレベルを持ち、その値は [`max_zoomlevel`](Self::max_zoomlevel)
    /// と一致します。値 `V` は各 `SingleId` に対応づけたまま返します。
    ///
    /// 空の木では空のイテレータを返します。
    ///
    /// # 例
    /// ```
    /// # use kasane_logic::{spatial_id::collection::flex_tree::core::FlexTreeCore, RangeId, SingleId};
    /// let mut core = FlexTreeCore::new();
    /// core.insert(SingleId::new(3, 3, 2, 7).unwrap(), 10);
    /// core.insert(RangeId::new(5, [1, 29], [8, 9], [5, 10]).unwrap(), 20);
    ///
    /// let max_z = core.max_zoomlevel().unwrap();
    /// let exported: Vec<_> = core.flat_single_ids().collect();
    ///
    /// assert!(exported.iter().all(|(single_id, _)| single_id.z() == max_z));
    /// ```
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, V)> {
        let Some(max_zoomlevel) = self.max_zoomlevel() else {
            return Vec::new().into_iter();
        };

        let mut exported = Vec::new();

        for (flex_id, value) in self.iter() {
            let range = RangeId::from(&flex_id);
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };

            for single_id in normalized.into_single_ids() {
                exported.push((single_id, value.clone()));
            }
        }

        exported.into_iter()
    }

    /// この [`FlexTreeCore`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として参照付きで書き出します。
    pub fn flat_single_ids_ref(&self) -> Box<dyn Iterator<Item = (SingleId, &V)> + '_> {
        let Some(max_zoomlevel) = self.max_zoomlevel() else {
            return Box::new(core::iter::empty());
        };

        Box::new(self.iter_ref().flat_map(move |(flex_id, value)| {
            let range = RangeId::from(&flex_id);
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };

            normalized
                .into_single_ids()
                .map(move |single_id| (single_id, value))
        }))
    }

    /// [FlexTreeCore]からtargetと重なりがある[FlexId]とそのValueへの参照を全て取り出す。
    pub fn get_ref<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds + 'a,
        V: 'a,
    {
        target.iter_flex_ids().flat_map(move |item| {
            self.overlap_ref(item.clone())
                .filter_map(move |(overlap_id, val)| {
                    overlap_id
                        .intersection(&item)
                        .map(|intersected_id| (intersected_id, val))
                })
        })
    }

    /// [FlexTreeCore]に空間IDを挿入する
    pub fn insert<S>(&mut self, target: S, value: V)
    where
        S: IterFlexIds,
    {
        for flex_id in target.iter_flex_ids() {
            if cfg!(not(feature = "temporal_id")) && !flex_id.temporal().is_whole() {
                panic!("TemporalIdはFlexTreeCoreに挿入できません。将来的に対応します。");
            }
            // シャード初期化されている場合、領域外は無視し、はみ出しは切り詰める。
            let flex_id = match &self.shard {
                Some(region) => match flex_id.intersection(region) {
                    Some(clipped) => clipped,
                    None => continue,
                },
                None => flex_id,
            };
            self.insert_flex_id(flex_id, value.clone());
        }
    }
    /// [FlexTreeCore]からtargetと重なりがある[FlexId]とそのValueを全て取り出す
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds + 'a,
        V: Clone + 'a,
    {
        target.iter_flex_ids().flat_map(move |item| {
            self.overlap(item.clone())
                .filter_map(move |(overlap_id, val)| {
                    overlap_id
                        // ここで安全に元の item を参照できる
                        .intersection(&item)
                        .map(|intersected_id| (intersected_id, val.clone()))
                })
        })
    }

    /// [FlexTreeCore]からTargetが示す領域を切り取って返す
    pub fn remove<S>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)>
    where
        S: IterFlexIds,
    {
        let mut actual_removed = Vec::new();

        for t_id in target.iter_flex_ids() {
            let affected_leaves: Vec<(FlexId, V)> = self.overlap_remove(&t_id).collect();

            for (leaf_id, value) in affected_leaves {
                for remnant_id in leaf_id.difference(&t_id) {
                    self.insert_flex_id(remnant_id, value.clone());
                }
                if let Some(intersect_id) = leaf_id.intersection(&t_id) {
                    actual_removed.push((intersect_id, value));
                }
            }
        }

        actual_removed.into_iter()
    }

    /// [`get`](Self::get) と同様に target と重なる要素を取り出しますが、
    /// **交差による切り取り（クリッピング）を行わず**、ツリーに格納されている
    /// [`FlexId`] をそのままの広さで返します。
    ///
    /// target が複数セルにまたがって同じ葉に複数回重なる場合でも、同一の葉は
    /// 1度だけ返します（重複除去）。
    pub fn get_overlapping<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds + 'a,
        V: Clone + 'a,
    {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for item in target.iter_flex_ids() {
            for (overlap_id, value) in self.overlap(item) {
                if seen.insert(overlap_id.clone()) {
                    results.push((overlap_id, value));
                }
            }
        }
        results.into_iter()
    }

    /// [`get_overlapping`](Self::get_overlapping) の参照版。値 `V` を複製せず参照で返します。
    pub fn get_overlapping_ref<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: IterFlexIds + 'a,
        V: 'a,
    {
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for item in target.iter_flex_ids() {
            for (overlap_id, value) in self.overlap_ref(item) {
                if seen.insert(overlap_id.clone()) {
                    results.push((overlap_id, value));
                }
            }
        }
        results.into_iter()
    }

    /// [`remove`](Self::remove) と異なり、**交差による切り取りや残余の再挿入を行わず**、
    /// target と少しでも重なった葉を丸ごとツリーから取り除き、その格納済み [`FlexId`] を
    /// そのままの広さで返します。
    pub fn remove_overlapping<S>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)>
    where
        S: IterFlexIds,
    {
        let mut removed = Vec::new();
        for t_id in target.iter_flex_ids() {
            removed.extend(self.overlap_remove(&t_id));
        }
        removed.into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返します。入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face_ref<'a, S>(
        &'a self,
        id: &S,
    ) -> alloc::vec::IntoIter<(FlexId, &'a V)>
    where
        S: SpatialId,
    {
        let self_ids: Vec<FlexId> = id.iter_flex_ids().collect();

        let mut slabs: Vec<S> = Vec::new();
        for delta in [-1, 1] {
            let mut sf = id.clone();
            if sf.move_f(delta).is_ok() {
                slabs.push(sf);
            }
            let mut sy = id.clone();
            if sy.move_y(delta).is_ok() {
                slabs.push(sy);
            }
            let mut sx = id.clone();
            sx.move_x(delta);
            slabs.push(sx);
        }

        let mut seen: HashSet<FlexId> = HashSet::new();
        let mut results: Vec<(FlexId, &'a V)> = Vec::new();

        for slab in &slabs {
            for slab_id in slab.iter_flex_ids() {
                for (cand, value) in self.overlap_ref(slab_id) {
                    if self_ids.iter().any(|s| cand.intersection(s).is_some()) {
                        continue;
                    }
                    if !self_ids.iter().any(|s| s.shares_face(&cand)) {
                        continue;
                    }
                    if seen.insert(cand.clone()) {
                        results.push((cand, value));
                    }
                }
            }
        }

        results.into_iter()
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueを取り出す
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        LeavesIter {
            stack: self.root_node_stack(),
        }
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueへの参照を取り出す。
    pub fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        LeavesIterRef {
            stack: self.root_node_stack(),
        }
    }

    /// 走査開始点として上下ルートノードを ID 付きで収集する。
    pub(super) fn root_node_stack(&self) -> Vec<(&Node<V>, FlexId)> {
        let mut stack = Vec::new();

        if !SharedNode::ptr_eq(&self.upper_root, &self.empty_leaf) {
            stack.push((self.upper_root.as_ref(), FlexId::UPPER_MAX));
        }

        if !SharedNode::ptr_eq(&self.lower_root, &self.empty_leaf) {
            stack.push((self.lower_root.as_ref(), FlexId::LOWER_MAX));
        }

        stack
    }

    /// 1つの FlexId を対応する上下ルートへ挿入する内部ユーティリティである。
    fn insert_flex_id(&mut self, flex_id: FlexId, value: V) {
        let root = if flex_id.f_index().is_negative() {
            &mut self.lower_root
        } else {
            &mut self.upper_root
        };

        Node::insert_mut(root, &flex_id, &value, 0, &self.empty_leaf);
    }
}

/// 軸と side に応じて、現在 ID から子ノード側の ID を1段分割して返す。
pub(super) fn split_child_id(current_id: &FlexId, axis: Dimension, side: Side) -> FlexId {
    match axis {
        Dimension::F => current_id.split_f(side).unwrap(),
        Dimension::X => current_id.split_x(side).unwrap(),
        Dimension::Y => current_id.split_y(side).unwrap(),
    }
}
