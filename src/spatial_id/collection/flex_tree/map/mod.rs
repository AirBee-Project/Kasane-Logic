use crate::IterSingleIds;
use alloc::vec::Vec;

use crate::spatial_id::collection::flex_tree::core::node_ops::TMapOverwrite;
use crate::{FlexId, FlexTreeCore, RangeId, SingleId, SpatialId, TemporalMap, TemporalSet};

pub mod convert;
pub mod json;
#[cfg(feature = "persist")]
pub mod persist;
pub mod shard;
pub mod tests;

/// 時空間(FlexId)に値(V)を対応づけるマップ構造。
///
/// 空間は木構造（FlexTree）の一次索引として、時間ごとの値は各空間セルの値
/// （[`TemporalMap<V>`]）として保持する（**時間ネイティブ**）。
/// 時間IDが全時間（WHOLE）のIDだけを扱う場合は、従来どおり純粋な空間マップとして振る舞う。
/// 挿入は後勝ち（同一時空間点は後から挿入した値で上書き）である。
#[derive(Default, Clone, Debug)]
pub struct SpatialIdMap<V>
where
    V: crate::spatial_id::collection::flex_tree::core::ptr::SafeValue,
{
    inner: FlexTreeCore<TemporalMap<V>>,
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

    /// 時空間に値を挿入します（後勝ち）。
    ///
    /// 時間付きの空間ID（temporal ≠ WHOLE）もそのまま受け付ける。既存と時空間が
    /// 重なる部分は新しい値で上書きされ、重ならない時間の値は保持される。
    pub fn insert<S: SpatialId>(&mut self, target: S, value: V) {
        for flex_id in target.iter_flex_ids() {
            // シャード領域外は無視し、はみ出しは切り詰める。
            let flex_id = match self.inner.shard() {
                Some(region) => match flex_id.intersection(region) {
                    Some(clipped) => clipped,
                    None => continue,
                },
                None => flex_id,
            };
            let temporal = flex_id.temporal().clone();
            let spatial = flex_id.spatial_part();
            let tmap = TemporalMap::from_temporal(&temporal, value.clone());
            if temporal.is_whole() {
                // 全時間の上書きは、覆う領域を置換する直接挿入と一致する。
                self.inner.insert_flex_id(spatial, tmap);
            } else {
                // 既存と時空間が重なる部分だけ b（新しい値）が勝つ。
                let mut single = FlexTreeCore::<TemporalMap<V>>::new();
                single.insert_flex_id(spatial, tmap);
                let shard = self.inner.shard().cloned();
                self.inner = self.inner.combine_with::<TMapOverwrite>(&single, shard);
            }
        }
    }

    /// 特定の時空間（target）と交差するすべての領域と、その値への参照を返します。
    ///
    /// 空間・時間の両方が target に切り取られる。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        S: SpatialId,
    {
        self.inner.get_ref(target).flat_map(|(clipped, tmap)| {
            tmap.cells_in_window_ref(clipped.temporal())
                .into_iter()
                .map(|(t, v)| (clipped.with_temporal(t), v))
                .collect::<Vec<_>>()
        })
    }

    /// 指定した時空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<S: SpatialId>(&mut self, target: &S) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed = Vec::new();
        for query in target.iter_flex_ids() {
            let q_spatial = query.spatial_part();
            let q_time = TemporalSet::from_temporal(query.temporal());
            // 空間的に重なる葉を丸ごと取り出し、残すべき部分を戻す。
            let affected: Vec<(FlexId, TemporalMap<V>)> =
                self.inner.remove_overlapping(&q_spatial).collect();
            for (leaf, tmap) in affected {
                // query の空間外の残余はそのまま戻す（キーは WHOLE 同士なので空間分割のみ）。
                for remnant in leaf.difference(&q_spatial) {
                    self.inner.insert_flex_id(remnant, tmap.clone());
                }
                // 空間交差部は時間で分割する。
                if let Some(inter) = leaf.intersection(&q_spatial) {
                    let kept = tmap.subtract_time(&q_time);
                    if !kept.is_empty() {
                        self.inner.insert_flex_id(inter.clone(), kept);
                    }
                    for (t, v) in tmap.intersect_time(&q_time).cells() {
                        removed.push((inter.with_temporal(t), v));
                    }
                }
            }
        }
        removed.into_iter()
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
        self.inner
            .get_overlapping_ref(target)
            .flat_map(|(stored, tmap)| {
                tmap.cells_ref()
                    .into_iter()
                    .map(|(t, v)| (stored.with_temporal(t), v))
                    .collect::<Vec<_>>()
            })
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と空間的に重なった
    /// 葉を（その全時間ごと）丸ごと取り除いて返します。
    pub fn remove_overlapping<S: SpatialId>(
        &mut self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, V)> {
        let mut removed = Vec::new();
        for (stored, tmap) in self.inner.remove_overlapping(target) {
            for (t, v) in tmap.cells() {
                removed.push((stored.with_temporal(t), v));
            }
        }
        removed.into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`]と値への参照を重複なく返します。
    /// 入力された空間ID自身と重なる要素は除外します。面共有の判定は空間3軸のみで行う。
    pub fn neighbors_share_face<'a, S: SpatialId>(
        &'a self,
        target: &S,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner
            .neighbors_share_face_ref(target)
            .flat_map(|(stored, tmap)| {
                tmap.cells_ref()
                    .into_iter()
                    .map(|(t, v)| (stored.with_temporal(t), v))
                    .collect::<Vec<_>>()
            })
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
    /// 各 [`SingleId`] には存在時間（時間セル）が付く。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        let max_zoomlevel = self.max_zoomlevel();
        self.iter().flat_map(move |(flex_id, value)| {
            let range = RangeId::from(&flex_id);
            let max_zoomlevel = max_zoomlevel.expect("non-empty iteration implies max zoom");
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
                .collect::<Vec<_>>()
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

    /// マップに保持されている全ての時空間と値への参照のペアを返します。
    ///
    /// 各空間セルの時間別の値は約数鎖の最小セル列へ分解され、
    /// `(空間セル × 時間セル, 値)` として列挙される。全時間（WHOLE）のセルは
    /// 従来どおり1つの `(FlexId, &V)` になる。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.iter_ref().flat_map(|(spatial, tmap)| {
            tmap.cells_ref()
                .into_iter()
                .map(|(t, v)| (spatial.with_temporal(t), v))
                .collect::<Vec<_>>()
        })
    }
}
