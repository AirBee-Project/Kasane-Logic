use crate::spatial_id::collection::temporal::SpatioTemporalCore;
use crate::{FlexId, RangeId, SingleId, SpatialId};

pub mod impls;

#[cfg(feature = "persist")]
pub mod persist;
pub mod shard;
pub mod tests;

/// 時空間(FlexId)に値(V)を対応づけるマップ構造。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    pub(crate) inner: SpatioTemporalCore<V>,
}

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::ptr::SafeValue,
{
    /// 空の [`SpatialIdMap`] を作成します。
    pub fn new() -> Self {
        Self {
            inner: SpatioTemporalCore::new(),
        }
    }

    /// シャード領域 `region` に閉じた空の[SpatialIdMap]を作成する。
    ///
    /// 以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        Self {
            inner: SpatioTemporalCore::new_in_shard(region),
        }
    }

    /// シャード領域を返す。`None` が帰ってきた場合はシャードされていない。
    pub fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }

    /// 時空間に値を挿入する。
    pub fn insert<S: SpatialId>(&mut self, target: S, value: V) {
        for flex_id in target.iter_flex_ids() {
            self.inner.insert_flex_id(flex_id, value.clone());
        }
    }

    /// 特定の時空間（target）と交差するすべての領域と、その値への参照を返す。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get(target)
    }

    /// 指定した時空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove(target)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と空間的に重なった
    /// [`FlexId`]と値への参照をそのまま返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId + 'a,
    {
        self.inner.get_overlapping(target)
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と空間的に重なった
    /// 葉を（その全時間ごと）丸ごと取り除いて返します。
    pub fn remove_overlapping<S: SpatialId>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove_overlapping(target)
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返します。
    /// 入力された空間ID自身と重なる要素は除外します。面共有の判定は空間3軸のみで行う。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.neighbors_share_face(target)
    }

    /// 保持している[FlexId]の総数を返します。
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// ツリーの最大ズームレベルを返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 最下層の[SingleId]レベルまで展開したイテレータを参照付きで返します。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        let max_zoomlevel = self.max_zoomlevel().unwrap_or(0);
        self.iter().flat_map(move |(flex_id, value)| {
            let range = RangeId::from(&flex_id);
            let normalized = if range.z() == max_zoomlevel {
                range
            } else {
                range
                    .spatial_children_at_zoom(max_zoomlevel)
                    .expect("target max zoomlevel must be valid")
            };
            normalized
                .single_ids()
                .map(move |single_id| (single_id, value))
        })
    }

    /// マップが空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 全てをクリアします。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// マップに保持されている全ての時空間と値への参照のペアを返す。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter()
    }
}
