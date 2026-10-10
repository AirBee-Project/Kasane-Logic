use crate::spatial_id::collection::flex_tree::core::{
    BitMask, FlexTreeCore, IntoIter, MinMax, Summary, ValueSet,
};
use core::ops::RangeBounds;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "rayon")]
pub mod par;
pub mod test;

use crate::{AllowedIntervals, FlexId, RangeId, SingleId, SpatialId};

/// 空間(FlexId)ごとに値(V)を持たせるためのテーブル構造。
///
/// `S` は木の Branch が子孫の値についてキャッシュする [Summary]。既定の [MinMax] は値の範囲で
/// 枝刈りする [`filter_range`](Self::filter_range) を使えるようにする。値で絞り込まないなら
/// [`NoSummary`](crate::NoSummary)、enum を種類で絞り込む [`filter_values`](Self::filter_values) を
/// 使うなら [`ValueSet`](crate::ValueSet) を選ぶ。
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
    pub fn get<'a, T: SpatialId<IntoIter: 'a>>(
        &'a self,
        target: T,
    ) -> impl Iterator<Item = (FlexId, &'a V)> + 'a {
        self.inner.get(target)
    }

    /// 指定した空間（target）をツリーからくり抜き、削除された領域とその値を返します。
    ///
    /// 削除は呼び出した時点で済んでおり、返すイテレーターはテーブルを借用しない。
    pub fn remove<T: SpatialId>(&mut self, target: T) -> IntoIter<V, S> {
        self.inner.remove(target).into_iter()
    }

    /// [`get`](Self::get) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのままの返します。
    pub fn get_overlapping<T: SpatialId>(
        &self,
        target: T,
    ) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.get_overlapping(target)
    }

    /// [`remove`](Self::remove) と異なり切り取りを行わず、target と重なった
    /// [`FlexId`]と値をそのまま取り除いて返します。
    ///
    /// 削除は呼び出した時点で済んでおり、返すイテレーターはテーブルを借用しない。
    pub fn remove_overlapping<T: SpatialId>(&mut self, target: T) -> IntoIter<V, S> {
        self.inner.remove_overlapping(target).into_iter()
    }

    /// 指定した単体の空間 IDと面で接している[`FlexId`] と値への参照を重複なく返します。入力された空間ID自身と重なる要素は除外します。
    pub fn neighbors_share_face<T: SpatialId>(
        &self,
        target: T,
    ) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.inner.neighbors_share_face(target)
    }

    /// 保持している[FlexId]の総数を返します。
    pub fn count(&self) -> usize {
        self.inner.count()
    }

    /// 保持している全ての[FlexId]のうち、最大のズームレベル値を返します。空なら [None]。
    ///
    /// キャッシュを持たず、全ての[FlexId]を走査する（O(n)）。
    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.inner.max_zoomlevel()
    }

    /// 時間方向に隣接する同値のSegmentを結合した [`RangeId`] と値を返す。**空間解像度は変えない**。
    ///
    /// `allowed_intervals` が [`None`] なら、各区間はそれを表せる最も粗い単位（`gcd(開始秒, 幅)`）の1Segmentになる。
    /// [`AllowedIntervals`] を渡すと、その候補のうち区間を割り切る最も粗い単位で表す。
    pub fn reconstructed_time_ranges<'a>(
        &'a self,
        allowed_intervals: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (RangeId, &'a V)> + 'a {
        self.inner.reconstructed_time_ranges(allowed_intervals)
    }

    /// [`range_ids`](Self::range_ids) を、テーブル全体の最大ズームレベルに揃えた [`SingleId`] へ展開する。
    pub fn flat_single_ids<'a>(
        &'a self,
        allowed_intervals: Option<&'a AllowedIntervals>,
    ) -> impl Iterator<Item = (SingleId, &'a V)> + 'a {
        self.inner.flat_single_ids(allowed_intervals)
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
    /// テーブル全体に存在する値の範囲 `(最小, 最大)`。空なら [`None`]。
    pub fn value_range(&self) -> Option<(&V, &V)> {
        self.inner.value_range()
    }

    /// 値が `range` に含まれる領域だけを残したテーブルを作る。値の範囲が `range` から外れる子孫は辿らない。
    pub fn filter_range<R: RangeBounds<V>>(&self, range: R) -> Self {
        Self {
            inner: self.inner.filter_range(range),
        }
    }
}

impl<V, S> SpatialIdTable<V, S>
where
    V: BitMask + PartialEq + Clone,
    S: Summary<V> + AsRef<ValueSet<V>>,
{
    /// テーブル全体に現れる値の集合。
    pub fn value_set(&self) -> ValueSet<V> {
        self.inner.value_set()
    }

    /// 値が `values` に含まれる領域だけを残したテーブルを作る。`values` と共通の値を持たない子孫は辿らない。
    pub fn filter_values(&self, values: ValueSet<V>) -> Self {
        Self {
            inner: self.inner.filter_values(values),
        }
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
