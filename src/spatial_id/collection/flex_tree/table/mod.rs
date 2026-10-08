use crate::spatial_id::collection::flex_tree::coalesce;
use crate::spatial_id::collection::flex_tree::core::{FlexTreeCore, IntoIter, MinMax, Summary};
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::ops::RangeBounds;
pub mod convert;
#[cfg(feature = "json")]
pub mod json;
pub mod test;

use crate::{AllowedIntervals, FlexId, RangeId, SingleId, SpatialId};

/// 空間(FlexId)ごとに値(V)を持たせるためのテーブル構造。
///
/// `S` は木の Branch が子孫の値についてキャッシュする [Summary]。既定の [MinMax] は値の範囲で
/// 枝刈りする [`value_range`](Self::value_range) などを速くする。値で絞り込まないなら
/// [`NoSummary`](crate::NoSummary)、enum を種類で絞り込むなら [`ValueSet`](crate::ValueSet) を選ぶ。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpatialIdTable<V, S = MinMax<V>> {
    pub(crate) inner: FlexTreeCore<V, S>,
}

impl<V, S> Default for SpatialIdTable<V, S> {
    fn default() -> Self {
        Self {
            inner: FlexTreeCore::default(),
        }
    }
}

impl<V: Ord + Clone> SpatialIdTable<V> {
    /// 既定の Summary（[MinMax]）を持つ空の[SpatialIdTable]を作成します。
    ///
    /// ほかの Summary を選ぶときは [`Default::default`] を使います。
    pub fn new() -> Self {
        Self::default()
    }
}

impl<V, S> SpatialIdTable<V, S>
where
    V: PartialEq + Clone,
    S: Summary<V>,
{
    /// 空間に値を挿入します。既に値がある場所は上書きされます。
    pub fn insert<T: SpatialId>(&mut self, target: T, value: V) {
        self.inner.insert(target, value);
    }

    /// まだ値の無い場所にだけ挿入します（Upsert）。既に値がある場所はそのまま保ちます。
    pub fn upsert<T: SpatialId>(&mut self, target: T, value: V) {
        self.inner
            .insert_with(target, value, |existing, _| existing.clone());
    }

    /// 指定した空間と重なる領域を、指定した空間との共通部分に切り取って値への参照と返します。
    pub fn get<'a, T>(&'a self, target: &'a T) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        T: SpatialId,
    {
        self.inner.get(target.clone())
    }

    /// 特定の範囲（RangeId）と交差するすべての領域と、その値への参照を返します。
    pub fn get_range<'a>(
        &'a self,
        target: &'a RangeId,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.get_overlapping_range(target)
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    pub fn remove<T: SpatialId>(&mut self, target: &T) -> Vec<(FlexId, V)> {
        self.inner.remove(target.clone()).into_iter().collect()
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのままの返します。
    pub fn get_overlapping<'a, T>(
        &'a self,
        target: &'a T,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a
    where
        T: SpatialId,
    {
        self.inner.get_overlapping(target.clone())
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのまま取り除いて返します。
    pub fn remove_overlapping<T: SpatialId>(&mut self, target: &T) -> Vec<(FlexId, V)> {
        self.inner
            .remove_overlapping(target.clone())
            .into_iter()
            .collect()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] と値への参照を重複なく返します。入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<'a, T: SpatialId>(
        &'a self,
        target: &T,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.neighbors_share_face(target)
    }

    /// 保持している[FlexId]の総数を返します。
    pub fn count(&self) -> usize {
        self.inner.len()
    }

    /// 保持している全ての[FlexId]のうち、最大のズームレベル値を返します。空なら [None]。
    ///
    /// キャッシュを持たず、全ての[FlexId]を走査する（O(n)）。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 時間方向に結合した [`RangeId`] として読み出す。**空間解像度は変えない**。
    ///
    /// 単位は「その区間を表せる最も粗い秒数」（`gcd(開始秒, 幅)`）。
    /// 単位を選びたい場合は [`range_ids_in`](Self::range_ids_in) を使う。
    pub fn range_ids(&self) -> impl Iterator<Item = (RangeId, &V)> + '_ {
        coalesce::range_ids(self.inner.iter(), None)
    }

    /// 時間の単位を [`AllowedIntervals`] の候補から選んで読み出す。
    ///
    /// 候補のうち**その区間を割り切る最も粗いもの**が選ばれる（＝候補の中でSegment数が最小）。
    /// 暦の単位へ正規化したいだけなら `AllowedIntervals::calendar()`
    /// （`temporal_id` feature 有効時のみ）を直接渡せる。
    pub fn range_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + 'a {
        coalesce::range_ids(self.inner.iter(), Some(units))
    }

    /// [`flat_single_ids`](Self::flat_single_ids) の、時間単位を指定できる版。
    pub fn flat_single_ids_in<'a>(
        &'a self,
        units: &'a AllowedIntervals,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + 'a {
        coalesce::range_ids(self.inner.iter(), Some(units))
            .flat_map(|(range, value)| range.single_ids().map(move |id| (id, value)))
    }

    /// 最下層の[SingleId]レベルまで展開したイテレータを参照付きで返します。
    ///
    /// 展開の前に、時間方向に隣接する同値のSegmentを結合する。木は時間を2の冪秒のSegmentとして
    /// 持つため、これを行わないと `1800` 秒のような単位で入れた ID が断片のまま出てくる。
    pub fn flat_single_ids(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        coalesce::range_ids(self.inner.iter(), None)
            .flat_map(|(range, value)| range.single_ids().map(move |id| (id, value)))
    }

    /// テーブルが空かどうかを返します
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// テーブルの全ての値を削除します。
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// テーブルに保持されている全ての空間と値への参照のペアを返します。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, &V)> + Clone + '_ {
        self.inner.iter()
    }
}

impl<V, S> SpatialIdTable<V, S>
where
    V: Ord + Clone,
    S: Summary<V> + AsRef<MinMax<V>>,
{
    /// 特定の値に対応するすべての[FlexId]を返す。
    pub fn value_get(&self, value: &V) -> impl Iterator<Item = FlexId> + use<V, S> {
        self.value_range(value..=value).map(|(flex_id, _)| flex_id)
    }

    /// 範囲条件に一致する全ての値の[FlexId]と値を返す。値の範囲が条件から外れる部分木は辿らない。
    pub fn value_range<R: RangeBounds<V>>(&self, range: R) -> IntoIter<V, S> {
        self.inner.filter_range(range).into_iter()
    }

    /// テーブルに保持されている値への参照を、重複なく昇順で返す。
    pub fn values(&self) -> impl Iterator<Item = &V> + '_ {
        self.inner
            .iter()
            .map(|(_, value)| value)
            .collect::<BTreeSet<_>>()
            .into_iter()
    }
}

impl<V: Clone, S> IntoIterator for SpatialIdTable<V, S> {
    type Item = (FlexId, V);
    type IntoIter = IntoIter<V, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> FromIterator<(FlexId, V)> for SpatialIdTable<V, S> {
    fn from_iter<T: IntoIterator<Item = (FlexId, V)>>(iter: T) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<V: PartialEq + Clone, S: Summary<V>> Extend<(FlexId, V)> for SpatialIdTable<V, S> {
    fn extend<T: IntoIterator<Item = (FlexId, V)>>(&mut self, iter: T) {
        self.inner.extend(iter);
    }
}
