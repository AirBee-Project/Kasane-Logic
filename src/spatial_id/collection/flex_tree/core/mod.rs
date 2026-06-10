use alloc::boxed::Box;
use alloc::vec::Vec;
use hashbrown::HashSet;

use crate::{
    Dimension, FlexId, IntoSingleIds, IterFlexIds, RangeId, Side, SingleId, SpatialId,
    spatial_id::collection::flex_tree::core::convert::{LeavesIter, LeavesIterRef},
};
use alloc::rc::Rc;
use node::Node;
mod convert;
pub mod node;
pub mod node_ops;
mod overlap;

/// 拡張空間IDとそれに紐づいたValueを保存するための型
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    lower_root: Rc<Node<V>>,
    upper_root: Rc<Node<V>>,
    empty_leaf: Rc<Node<V>>,
}

impl<V> Default for FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    /// 新しい空の[FlexTreeCore]を作成する
    pub fn new() -> Self {
        let empty_leaf = Rc::new(Node::Leaf { value: None });
        Self {
            lower_root: empty_leaf.clone(),
            upper_root: empty_leaf.clone(),
            empty_leaf,
        }
    }

    /// 2つの [FlexTreeCore] の和集合（Union）を計算します。
    ///
    /// 値が設定されている（`Some`）領域は基本的に保持されますが、`other` がより粗い領域で `Some` を持つ場合、その領域では `self` のより細かい値が上書きされることがあります（両者が同じ `Leaf(Some)` の場合のみ `self` が優先）。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            lower_root: Node::union(&self.lower_root, &other.lower_root, 0, &self.empty_leaf),
            upper_root: Node::union(&self.upper_root, &other.upper_root, 0, &self.empty_leaf),
            empty_leaf: self.empty_leaf.clone(),
        }
    }

    /// 2つの [FlexTreeCore] の積集合（Intersection）を計算します。
    ///
    /// 両方の木で値が設定されている（重なり合う）領域については、以下のように値が保持されます。
    /// - `self` と `other` で階層（細かさ）が異なる場合、**より細かい領域（より深い階層）で定義されている側の値**が優先して保持されます。
    /// - 両者が同じ広さ（階層）で完全に一致する場合、**引数（`other`）の値**が優先して保持されます。
    pub fn intersection(&self, other: &Self) -> Self {
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
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        Self {
            lower_root: Node::difference(&self.lower_root, &other.lower_root, 0, &self.empty_leaf),
            upper_root: Node::difference(&self.upper_root, &other.upper_root, 0, &self.empty_leaf),
            empty_leaf: self.empty_leaf.clone(),
        }
    }

    /// ルートノードのRcポインタが完全に同一か判定します（Result Reuseテスト用）
    #[cfg(test)]
    pub fn root_ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.lower_root, &other.lower_root)
            && Rc::ptr_eq(&self.upper_root, &other.upper_root)
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

    /// この集合が値を持つ全セルを包む最小の範囲（F/X/Y の3次元AABB）を返します。
    ///
    /// 返り値 [`RangeId`] の各次元の `[0]` が最小（左下）側、`[1]` が最大（右上）側の角に
    /// 対応します。混在ズームのセルは木全体の最大ズームへ正規化したうえで比較されます。
    /// 空の木では [`None`] を返します。
    ///
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

        if !Rc::ptr_eq(&self.upper_root, &self.empty_leaf) {
            stack.push((self.upper_root.as_ref(), FlexId::UPPER_MAX));
        }

        if !Rc::ptr_eq(&self.lower_root, &self.empty_leaf) {
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
