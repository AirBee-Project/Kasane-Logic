use crate::spatial_id::collection::flex_tree::core::FlexTreeCore;
use crate::spatial_id::collection::flex_tree::shard_path::ShardPath;
use crate::spatial_id::collection::flex_tree::summary::ShardSummary;
use crate::{AllowedIntervals, FlexId, RangeId, SingleId, SpatialId};
use alloc::vec::Vec;

#[cfg(feature = "persist")]
pub mod archived;
#[cfg(feature = "persist")]
pub mod arena;
pub mod convert;
#[cfg(feature = "json")]
pub mod json;
pub mod shard;
pub mod tests;

/// 空間(FlexId)に値(V)を対応づける、可変な作業構造。
///
/// `Arc`/`Rc` ベースの永続木（`FlexTreeCore`）を包み、構造共有・COW・並列集合演算の恩恵を
/// そのまま持つ。バイト列との変換や、それを再構築せず直接読む方法は次を参照（`persist` feature）：
///
/// - `to_bytes` / `from_bytes`：このバイト列表現との相互変換。
/// - `ArchivedSpatialIdMap`：`to_bytes` が書いたバイト列を、
///   この型へ再構築せず直接読む読み取り専用のゼロコピービュー。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    inner: FlexTreeCore<V>,
}

impl<V> PartialEq for SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<V> Eq for SpatialIdMap<V> where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue
{
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

    /// この集合が値を持つ全Segmentを包む最小の[RangeId]を返します。
    pub fn bounding_box(&self) -> Option<RangeId> {
        self.inner.bounding_box()
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
        self.inner.get_ref(target.clone())
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> Vec<(FlexId, V)> {
        self.inner.remove(target.clone())
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
        self.inner.get_overlapping_ref(target.clone())
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのまま取り除いて返します。
    pub fn remove_overlapping<S: SpatialId>(&mut self, target: &S) -> Vec<(FlexId, V)> {
        self.inner.remove_overlapping(target.clone())
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

    /// このマップを、木を走査せずに評価するための要約を作ります。
    ///
    /// `persist` feature が有効なら `to_bytes` がこれをバイト列へ焼き、
    /// `ArchivedSpatialIdMap::summary` が木を復元せずに読み出せます。
    pub fn summary(&self) -> ShardSummary {
        self.inner.summary()
    }

    /// このマップのシャード木上の位置。定まらない場合は [`None`]。
    pub fn shard_path(&self) -> Option<&ShardPath> {
        self.inner.shard_path()
    }

    /// ツリーの最大ズームレベルを返します。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 時間方向に結合した [`RangeId`] として読み出す。**空間解像度は変えない**。
    ///
    /// 単位は「その区間を表せる最も粗い秒数」（`gcd(開始秒, 幅)`）。
    /// 単位を選びたい場合は [`range_ids_in`](Self::range_ids_in) を使う。
    pub fn range_ids(&self) -> impl Iterator<Item = (RangeId, &V)> + '_ {
        self.inner.range_ids_ref(None)
    }

    /// 時間の単位を [`AllowedIntervals`] の候補から選んで読み出す。
    ///
    /// 候補のうち**その区間を割り切る最も粗いもの**が選ばれる（＝候補の中でSegment数が最小）。
    /// 暦の単位へ正規化したいだけなら `AllowedIntervals::calendar()`
    /// （`temporal_id` feature 有効時のみ）を直接渡せる。
    pub fn range_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + use<'a, V> {
        self.inner.range_ids_ref(Some(units))
    }

    /// [`flat_single_ids`](Self::flat_single_ids) の、時間単位を指定できる版。
    pub fn flat_single_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + use<'a, V> {
        self.inner.flat_single_ids_in_ref(Some(units))
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
