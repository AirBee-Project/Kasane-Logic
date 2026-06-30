use crate::{FlexId, FlexTreeCore, SingleId, SpatialId};

pub mod convert;
pub mod json;
#[cfg(feature = "persist")]
pub mod persist;
pub mod shard;
pub mod tests;

/// 空間(FlexId)に値(V)を対応づけるマップ構造。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    inner: FlexTreeCore<V>,
}

impl<V> SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    /// 空の [`SpatialIdMap`] を作成します。
    pub fn new() -> Self {
        Self {
            inner: FlexTreeCore::default(),
        }
    }

    /// シャード領域 `region` に閉じた空の[SpatialIdMap]を作成する。
    ///
    /// 以降は `region` の内側だけを保持する。`region` の外側への挿入は無視される。
    pub fn new_in_shard(region: FlexId) -> Self {
        Self {
            inner: FlexTreeCore::new_in_shard(region),
        }
    }

    /// シャード領域を返す。`None` が帰ってきた場合はシャードされていない。
    pub fn shard(&self) -> Option<&FlexId> {
        self.inner.shard()
    }

    /// 空間に値を挿入します。
    pub fn insert<S: SpatialId>(&mut self, target: S, value: V) {
        self.inner.insert(target, value);
    }

    /// 特定の空間（target）と交差するすべての領域と、その値への参照を返します。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get_ref(target)
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove(target)
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値への参照をそのまま返します。
    pub fn get_overlapping<'a, S>(
        &'a self,
        target: &'a S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId + 'a,
    {
        self.inner.get_overlapping_ref(target)
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのまま取り除いて返します。
    pub fn remove_overlapping<S: SpatialId>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        self.inner.remove_overlapping(target)
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返します。
    /// 入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.neighbors_share_face_ref(target)
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
        self.inner.flat_single_ids_ref()
    }

    /// マップが空かどうかを返します。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 全てをクリアします。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// マップに保持されている全ての空間と値への参照のペアを返します。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter_ref()
    }
}
