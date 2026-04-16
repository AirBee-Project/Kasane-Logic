use crate::{
    Dimension, FlexId, IntoSingleIds, IterFlexIds, RangeId, Side, SingleId,
    spatial_id::collection::flex_tree::core::convert::LeavesIter,
};
use node::Node;
mod convert;
mod node;
mod overlap;

/// 拡張空間IDとそれに紐づいたValueを保存するための型
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    lower_root: Option<Box<Node<V>>>,
    upper_root: Option<Box<Node<V>>>,
}

impl<V> FlexTreeCore<V>
where
    V: PartialEq + Clone,
{
    /// 新しい空の[FlexTreeCore]を作成する
    pub fn new() -> Self {
        Self {
            lower_root: None,
            upper_root: None,
        }
    }

    ///クリアする
    pub fn clear(&mut self) {
        self.lower_root = None;
        self.upper_root = None;
    }

    pub fn is_empty(&self) -> bool {
        self.lower_root.is_none() && self.upper_root.is_none()
    }

    pub fn count(&self) -> usize {
        //Todo:型の内部に個数をキャッシュしてO(1)で求められるようにする
        self.iter().count()
    }

    /// この [`FlexTreeCore`] に含まれる要素のうち、最も高いズームレベル値を返します。
    ///
    /// ここでいう解像度は、各 [`FlexId`] の `f/x/y` それぞれのズームレベルの最大値です。
    /// 空の木では [`None`] を返します。
    ///
    /// # 例
    /// ```
    /// # use kasane_logic::{FlexTreeCore, RangeId};
    /// let mut core = FlexTreeCore::new();
    /// core.insert(RangeId::new(4, [0, 1], [0, 0], [0, 0]).unwrap(), ());
    /// assert_eq!(core.max_zoomlevel(), Some(4));
    /// ```
    pub fn max_zoomlevel(&self) -> Option<u8> {
        //Todo:全探索にならない実装をしたほうが良い
        self.iter()
            .map(|(flex_id, _)| {
                flex_id
                    .f_zoomlevel()
                    .max(flex_id.x_zoomlevel())
                    .max(flex_id.y_zoomlevel())
            })
            .max()
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
    /// # use kasane_logic::{FlexTreeCore, RangeId, SingleId};
    /// let mut core = FlexTreeCore::new();
    /// core.insert(SingleId::new(3, 3, 2, 7).unwrap(), 10);
    /// core.insert(RangeId::new(5, [1, 29], [8, 9], [5, 10]).unwrap(), 20);
    ///
    /// let max_z = core.max_zoomlevel().unwrap();
    /// let exported: Vec<_> = core.flat_single_ids().collect();
    ///
    /// assert!(exported.iter().all(|(single_id, _)| single_id.z() == max_z));
    /// ```
    pub fn flat_single_ids(&self) -> std::vec::IntoIter<(SingleId, V)> {
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

    /// [FlexTreeCore]に空間IDを挿入する
    pub fn insert<S>(&mut self, target: S, value: V)
    where
        S: IterFlexIds,
    {
        for flex_id in target.iter_flex_ids() {
            let root = match flex_id.f_index().is_negative() {
                true => &mut self.lower_root,
                false => &mut self.upper_root,
            };

            if root.is_none() {
                *root = Some(Box::new(Node::Branch {
                    axis: Dimension::F,
                    lower_child: None,
                    upper_child: None,
                }));
            }

            if let Some(kd_node) = root {
                kd_node.insert(flex_id, value.clone(), 0, 0, 0);
            }
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
                    self.insert(remnant_id, value.clone());
                }
                if let Some(intersect_id) = leaf_id.intersection(&t_id) {
                    actual_removed.push((intersect_id, value));
                }
            }
        }

        actual_removed.into_iter()
    }

    /// [FlexTreeCore]から全ての[FlexId]とValueを取り出す
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        LeavesIter {
            stack: self.root_node_stack(),
        }
    }

    /// 走査開始点として上下ルートノードを ID 付きで収集する。
    pub(super) fn root_node_stack(&self) -> Vec<(&Node<V>, FlexId)> {
        let mut stack = Vec::new();

        if let Some(upper) = &self.upper_root {
            stack.push((upper.as_ref(), FlexId::UPPER_MAX));
        }

        if let Some(lower) = &self.lower_root {
            stack.push((lower.as_ref(), FlexId::LOWER_MAX));
        }

        stack
    }
}

/// 軸と side に応じて、現在 ID から子ノード側の ID を1段分割して返す。
pub(super) fn split_child_id(current_id: &FlexId, axis: Dimension, side: Side) -> FlexId {
    match axis {
        Dimension::F => current_id.f_split(side).unwrap(),
        Dimension::X => current_id.x_split(side).unwrap(),
        Dimension::Y => current_id.y_split(side).unwrap(),
    }
}
