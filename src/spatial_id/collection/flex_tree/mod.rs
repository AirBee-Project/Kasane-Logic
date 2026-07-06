#![allow(dead_code)]
use crate::{IterFlexIds, IterSingleIds};
use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::{
    Dimension, FlexId, RangeId, Side, SingleId, SpatialId,
    spatial_id::collection::flex_tree::convert::{LeavesIter, LeavesIterRef},
};
use node::Node;
mod convert;
pub mod node;
pub mod node_ops;
mod overlap;
pub(crate) mod ptr;
pub mod shard;
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
pub(crate) struct FlexTree<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    pub(crate) lower_root: SharedNode<Node<V>>,
    pub(crate) upper_root: SharedNode<Node<V>>,
    pub(crate) empty_leaf: SharedNode<Node<V>>,

    /// シャード空間の有無。
    pub(crate) shard: Option<FlexId>,
}

impl<V> Default for FlexTree<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PartialEq for FlexTree<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    fn eq(&self, other: &Self) -> bool {
        self.lower_root == other.lower_root && self.upper_root == other.upper_root
    }
}

impl<V> Eq for FlexTree<V> where V: crate::spatial_id::collection::flex_tree::ptr::SafeValue {}

impl<V> FlexTree<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    /// 新しい空の[FlexTree]を作成する
    pub fn new() -> Self {
        let empty_leaf = SharedNode::new(Node::Leaf { value: None });
        Self {
            lower_root: empty_leaf.clone(),
            upper_root: empty_leaf.clone(),
            empty_leaf,
            shard: None,
        }
    }

    /// シャード領域 `region` に閉じた空の[FlexTree]を作成する。以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        let mut core = Self::new();
        core.shard = Some(region);
        core
    }

    /// このツリーが閉じているシャード領域を返す。`None` は全空間。
    pub(crate) fn shard(&self) -> Option<&FlexId> {
        self.shard.as_ref()
    }

    /// 2つの [FlexTree] の和集合を計算します。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            lower_root: Node::union(&self.lower_root, &other.lower_root, 0, &self.empty_leaf),
            upper_root: Node::union(&self.upper_root, &other.upper_root, 0, &self.empty_leaf),
            empty_leaf: self.empty_leaf.clone(),
            shard: Self::shard_after_union(&self.shard, &other.shard),
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
                shard: Self::shard_after_intersection(&self.shard, &other.shard),
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
            shard: Self::shard_after_intersection(&self.shard, &other.shard),
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

    /// 値結合 [`Combine`](node_ops::Combine) を差し込んだ汎用の二項演算。
    /// union/intersection/difference を値付き（時間集合など）で行うためのネイティブ経路。
    ///
    /// `shard` には結果のシャード領域を渡す（union なら
    /// [`shard_after_union`](Self::shard_after_union) 相当、difference なら
    /// `self.shard` など、演算に応じて呼び出し側が決める）。
    pub(crate) fn combine_with<C: node_ops::Combine<V>>(
        &self,
        other: &Self,
        shard: Option<FlexId>,
    ) -> Self {
        Self {
            lower_root: Node::combine::<C>(
                &self.lower_root,
                &other.lower_root,
                0,
                &self.empty_leaf,
            ),
            upper_root: Node::combine::<C>(
                &self.upper_root,
                &other.upper_root,
                0,
                &self.empty_leaf,
            ),
            empty_leaf: self.empty_leaf.clone(),
            shard,
        }
    }

    /// unionのシャード合成規則を公開ヘルパとして提供する（値結合経路用）。
    pub(crate) fn shard_for_union(a: &Self, b: &Self) -> Option<FlexId> {
        Self::shard_after_union(&a.shard, &b.shard)
    }

    /// intersectionのシャード合成規則を公開ヘルパとして提供する（値結合経路用）。
    pub(crate) fn shard_for_intersection(a: &Self, b: &Self) -> Option<FlexId> {
        Self::shard_after_intersection(&a.shard, &b.shard)
    }

    /// ルートノードのポインタが完全に同一か判定します（Result Reuseテスト用）
    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        SharedNode::ptr_eq(&self.lower_root, &other.lower_root)
            && SharedNode::ptr_eq(&self.upper_root, &other.upper_root)
    }

    /// コレクション内のすべての値をインプレースで更新します。
    pub fn map_values_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut V),
    {
        SharedNode::make_mut(&mut self.lower_root).map_values_mut(&mut f);
        SharedNode::make_mut(&mut self.upper_root).map_values_mut(&mut f);
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

    /// この [`FlexTree`] に含まれる要素のうち、最も高いズームレベル値を返します。ここでいう解像度は、各 [`FlexId`] の `f/x/y` それぞれのズームレベルの最大値です。
    /// 空の木では [`None`] を返します。
    ///
    /// # 例
    /// ```ignore
    /// # use kasane_logic::{spatial_id::collection::tree::FlexTree, RangeId, SingleId};
    /// let mut core = FlexTree::new();
    /// core.insert(RangeId::new(4, [0, 1], [0, 0], [0, 0]).unwrap(), ());
    /// assert_eq!(core.max_zoomlevel(), Some(4));
    /// ```ignore
    pub fn max_zoomlevel(&self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }
        let lower = self.lower_root.max_zoom_at(0);
        let upper = self.upper_root.max_zoom_at(0);
        Some(lower.max(upper))
    }

    /// この集合が値を持つ全セルを包む最小の[RangeId]を返します。
    /// # 例
    /// ```ignore
    /// # use kasane_logic::{spatial_id::collection::tree::FlexTree, SingleId};
    /// let mut core = FlexTree::new();
    /// core.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
    /// core.insert(SingleId::new(20, 0, 2, 3).unwrap(), 1);
    ///
    /// let bbox = core.bounding_box().unwrap();
    /// assert_eq!(bbox.z(), 20);
    /// assert_eq!(bbox.f(), [0, 0]);
    /// assert_eq!(bbox.x(), [0, 2]);
    /// assert_eq!(bbox.y(), [0, 3]);
    ///
    /// let empty: FlexTree<i32> = FlexTree::new();
    /// assert!(empty.bounding_box().is_none());
    /// ```ignore
    pub fn bounding_box(&self) -> Option<RangeId> {
        RangeId::bounding_box_of(self.iter().map(|(flex_id, _)| flex_id))
    }

    /// この [`FlexTree`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として書き出す。
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

            for single_id in normalized.iter_single_ids() {
                exported.push((single_id, value.clone()));
            }
        }

        exported.into_iter()
    }

    /// この [`FlexTree`] に含まれる要素を、木全体の `max_zoomlevel` に揃えた [`SingleId`] として値の参照付きで書き出す。
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
                .iter_single_ids()
                .collect::<alloc::vec::Vec<_>>()
                .into_iter()
                .map(move |single_id| (single_id, value))
        }))
    }

    /// [FlexTree]からtargetと重なりがある[FlexId]とそのValueへの参照を全て取り出す。
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

    /// [FlexTree]に空間IDを挿入する。
    ///
    /// # Panics
    ///
    /// コレクションは [`SpatialIdSet`](crate::SpatialIdSet) である。
    pub fn insert<S>(&mut self, target: S, value: V)
    where
        S: IterFlexIds,
    {
        for flex_id in target.iter_flex_ids() {
            if !flex_id.temporal().is_whole() {
                panic!("FlexTree does not support temporal IDs.");
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

    /// [FlexTree]からtargetと重なりがある[FlexId]とそのValueを全て取り出す
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds + 'a,
        V: Clone + 'a,
    {
        target.iter_flex_ids().flat_map(move |item| {
            self.overlap(item.clone())
                .filter_map(move |(overlap_id, val)| {
                    overlap_id
                        .intersection(&item)
                        .map(|intersected_id| (intersected_id, val.clone()))
                })
        })
    }

    /// [FlexTree]からTargetが示す領域を削除して、返す。
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
    /// 切り取りを行わず、[`FlexId`] をそのままの広さで返す。
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

    /// [`get_overlapping`](Self::get_overlapping) の参照版。
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

    /// [`remove`](Self::remove) と異なり、**交差による切り取りや残余の再挿入を行わず**、 target と少しでも重なった葉を丸ごとツリーから取り除き、その格納済み [`FlexId`] を そのままの広さで返す。
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

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返す。入力された空間ID自身と重なる要素は除外する。
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

    /// [FlexTree]から全ての[FlexId]とValueを取り出す
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        LeavesIter {
            stack: self.root_node_stack(),
        }
    }

    /// [FlexTree]から全ての[FlexId]とValueへの参照を取り出す。
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

    /// 1つの [`FlexId`]（空間キー）へ値を直接書き込む（覆う領域は置換）。
    ///
    /// 時間ネイティブなラッパー（[`SpatialIdSet`](crate::SpatialIdSet) など）が、
    /// 時間を値へ移し替えた上でキー（temporal=WHOLE）を挿入する経路。
    pub(crate) fn insert_flex_id(&mut self, flex_id: FlexId, value: V) {
        let root = if flex_id.f_index().is_negative() {
            &mut self.lower_root
        } else {
            &mut self.upper_root
        };
        Node::insert_mut(root, &flex_id, &value, 0, &self.empty_leaf);
    }

    /// unionのシャード領域を返す。
    /// シャードされている場合とされていない場合があるので、そのラッパー
    fn shard_after_union(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
        match (a, b) {
            (Some(a), Some(b)) if a == b => Some(a.clone()),
            _ => None,
        }
    }

    /// intersectionのシャード領域を返す。
    /// シャードされている場合とされていない場合があるので、そのラッパー
    fn shard_after_intersection(a: &Option<FlexId>, b: &Option<FlexId>) -> Option<FlexId> {
        match (a, b) {
            (Some(a), Some(b)) => a.intersection(b).or_else(|| Some(a.clone())),
            (Some(a), None) => Some(a.clone()),
            (None, Some(b)) => Some(b.clone()),
            (None, None) => None,
        }
    }
}

/// 軸と side に応じて、現在 ID から子ノード側の ID を1段分割して返す。
pub(crate) fn split_child_id(current_id: &FlexId, axis: Dimension, side: Side) -> FlexId {
    match axis {
        Dimension::F => current_id.split_f(side).unwrap(),
        Dimension::X => current_id.split_x(side).unwrap(),
        Dimension::Y => current_id.split_y(side).unwrap(),
    }
}
