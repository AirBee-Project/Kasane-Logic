//! 木から読み出したSegment列を、時間方向に結合してから書き出すための層。
//!
//! FlexTree は時間軸を「2の冪秒の2分岐Segment」として持つため、`Interval` が2の冪でない時間 ID
//! （仕様が認める `1800` 秒など）は挿入時に複数Segmentへ分解される。読み出しでSegmentを1つずつ
//! そのまま返すと、`12/0/3638/1614_1800/809712` が5つの断片になってしまい、
//! 仕様書の `{i}/{t}` 表記が失われる。
//!
//! ここでは「FlexIdと値が同じで、時間が隣接している」Segmentどうしを結合してから
//! `RangeId::with_time_span` に渡す。結合後の秒区間は元の区間に戻るので、
//! `gcd` ベースの復元によって `1800/809712` がそのまま取り出せる。
//!
//! 結合は [`FlexId`] では表現できない（[`FlexId`] は2分岐Segment1個しか持てない）ため、
//! 出力は [`RangeId`] である。したがってこの層は「[`RangeId`] / [`SingleId`] を書き出す経路」
//! （`flat_single_ids` と JSON 直列化）専用で、`get` / `iter` のような [`FlexId`] を返す
//! 経路は従来どおり生のSegmentを返す。

use crate::SpatialId;
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{AllowedIntervals, FlexId, RangeId};

/// 空間成分のみを `u128` へパックし、高速な一致判定を行う。
#[inline(always)]
fn spatial_key_u128(id: &FlexId) -> u128 {
    ((id.f_zoomlevel() as u128) << 120)
        | ((id.f_index() as u128) << 96)
        | ((id.x_zoomlevel() as u128) << 88)
        | ((id.x_index() as u128) << 64)
        | ((id.y_zoomlevel() as u128) << 56)
        | (id.y_index() as u128)
}

/// 時間方向に隣接するSegmentを遅延評価で結合するイテレータ。
pub struct CoalesceTemporal<'a, I, V>
where
    I: Iterator<Item = (FlexId, V)>,
    V: PartialEq,
{
    iter: I,
    pending: Option<(u128, FlexId, u64, u64, V)>,
    units: Option<&'a AllowedIntervals>,
}

impl<'a, I, V> Iterator for CoalesceTemporal<'a, I, V>
where
    I: Iterator<Item = (FlexId, V)>,
    V: PartialEq,
{
    type Item = (RangeId, V);

    fn next(&mut self) -> Option<Self::Item> {
        let (key, first_flex, start, mut end, value) = match self.pending.take() {
            Some(p) => p,
            None => {
                let (id, v) = self.iter.next()?;
                let (s, e) = id.seconds_range();
                (spatial_key_u128(&id), id, s, e, v)
            }
        };

        for (next_id, next_value) in self.iter.by_ref() {
            let next_key = spatial_key_u128(&next_id);
            let (next_start, next_end) = next_id.seconds_range();

            if next_key == key && next_value == value && next_start == end {
                end = next_end;
                continue;
            }

            self.pending = Some((next_key, next_id, next_start, next_end, next_value));
            return Some(finish(&first_flex, start, end, value, self.units));
        }

        Some(finish(&first_flex, start, end, value, self.units))
    }
}

/// 時間方向に隣接するSegmentを結合し、[`RangeId`] の列として返す。
///
/// `units` に[`AllowedIntervals`]を渡すと、結合後の秒区間を**その候補のうち最も粗い単位**で
/// 表し直す（Segment数が候補の中で最小になる）。`None` の場合は「その区間を表せる最も粗い単位」
/// （`gcd(開始秒, 幅)`）になり、Segment数は常に1になる。
///
/// [`AllowedIntervals`] は必ず全区間を表せる候補を含むので、**この関数は失敗しない**。
///
/// # 計算量
///
/// `O(n)`。入力はあらかじめ `(FlexId, 開始秒)` 順に並んでいる前提。
pub(crate) fn coalesce_temporal<'a, I, V>(
    rows: I,
    units: Option<&'a AllowedIntervals>,
) -> impl Iterator<Item = (RangeId, V)> + 'a
where
    I: IntoIterator<Item = (FlexId, V)> + 'a,
    V: PartialEq + 'a,
{
    CoalesceTemporal {
        iter: rows.into_iter(),
        pending: None,
        units,
    }
}

/// 結合し終えた1件を [`RangeId`] に組み立てる。
fn finish<V>(
    key: &FlexId,
    start: u64,
    end: u64,
    value: V,
    units: Option<&AllowedIntervals>,
) -> (RangeId, V) {
    let range = RangeId::from(key)
        .with_time_span(start, end)
        .expect("結合した秒区間は元のSegmentの和なので常に有効");

    // 候補集合が指定されていれば、その中で最も粗い（＝Segment数が最小の）単位へ表し直す。
    // `AllowedIntervals` は必ず全区間を表せる候補を含むので、この `relabel_time` は失敗しない。
    let range = match units {
        None => range,
        Some(units) => range
            .clone()
            .relabel_time(units.coarsest_dividing(start, end))
            .expect("AllowedIntervals が選んだ単位は必ずこの区間を割り切る"),
    };

    (range, value)
}

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;
    use crate::{Interval, SingleId, SpatialId};

    fn coalesce_temporal_vec<V: Clone + PartialEq + Send>(
        items: impl IntoIterator<Item = (FlexId, V)>,
        units: Option<&AllowedIntervals>,
    ) -> Vec<(RangeId, V)> {
        let mut items: Vec<_> = items.into_iter().collect();
        items.sort_by(|a, b| {
            let key_a = spatial_key_u128(&a.0);
            let key_b = spatial_key_u128(&b.0);
            key_a
                .cmp(&key_b)
                .then_with(|| a.0.seconds_range().0.cmp(&b.0.seconds_range().0))
        });
        coalesce_temporal(items, units).collect()
    }

    /// 時間成分を全時間へ落とした [`FlexId`]。元のテストの `sorted_reference` 用。
    #[allow(dead_code)]
    pub(crate) fn spatial_key(flex_id: &FlexId) -> FlexId {
        flex_id.with_time_segment(0, 0)
    }

    /// 仕様書 1.5.3 の例（`1800/809712`）が、分解 → 結合を経て元の表記へ戻る。
    #[test]
    fn round_trips_the_spec_example() {
        let original = SingleId::new(12, 0, 3638, 1614)
            .unwrap()
            .with_time(1800, 809712)
            .unwrap();

        let time_segments: Vec<_> = original.clone().into_iter().map(|id| (id, 1u8)).collect();
        assert!(
            time_segments.len() > 1,
            "1800秒は複数の2分岐Segmentへ分解されるはず"
        );

        let merged = coalesce_temporal_vec(time_segments, None);
        assert_eq!(merged.len(), 1, "結合されて1件になるはず");
        assert_eq!(merged[0].0.to_string(), "12/0/3638/1614_1800/809712");
    }

    /// 値が違うSegmentは結合しない。
    #[test]
    fn does_not_merge_across_different_values() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut time_segments: Vec<(FlexId, u8)> = Vec::new();
        time_segments.extend(a.into_iter().map(|id| (id, 1u8)));
        time_segments.extend(b.into_iter().map(|id| (id, 2u8)));

        let merged = coalesce_temporal_vec(time_segments, None);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].0.seconds_range(), (0, 3600));
        assert_eq!(merged[1].0.seconds_range(), (3600, 7200));
    }

    /// 同じ値なら、隣接する2つの時間Segmentは1つに融合する。
    #[test]
    fn merges_adjacent_segments_with_the_same_value() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut time_segments: Vec<(FlexId, u8)> = Vec::new();
        time_segments.extend(a.into_iter().map(|id| (id, 7u8)));
        time_segments.extend(b.into_iter().map(|id| (id, 7u8)));

        let merged = coalesce_temporal_vec(time_segments, None);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0.seconds_range(), (0, 7200));
        // gcd(0, 7200) = 7200 なので「2時間」という単位で1Segmentになる。
        assert_eq!(
            (merged[0].0.time_interval().seconds(), merged[0].0.t()),
            (7200, [0, 0])
        );
    }

    /// 候補集合を渡すと、その中で最も粗い（＝Segment数が最小の）単位が選ばれる。
    #[test]
    fn candidate_set_picks_the_coarsest_unit() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut time_segments: Vec<(FlexId, u8)> = Vec::new();
        time_segments.extend(a.into_iter().map(|id| (id, 7u8)));
        time_segments.extend(b.into_iter().map(|id| (id, 7u8)));

        let merged = coalesce_temporal_vec(time_segments, Some(AllowedIntervals::calendar()));
        assert_eq!(merged.len(), 1);
        assert_eq!(
            (merged[0].0.time_interval().seconds(), merged[0].0.t()),
            (3600, [0, 1])
        );

        // 候補が区間を割り切れないときは、必ず含まれる SECOND まで落ちる。
        // 候補集合が貧しいとSegment数が爆発することを示す例でもある
        // （1時間 = 3600 TimeSegment）。実用では `AllowedIntervals::calendar()` のように
        // 粒度の階段を用意しておくこと。
        let mut time_segments: Vec<(FlexId, u8)> = Vec::new();
        time_segments.extend(
            SingleId::new(12, 0, 3638, 1614)
                .unwrap()
                .with_time(Interval::HOUR, 0)
                .unwrap()
                .into_iter()
                .map(|id| (id, 7u8)),
        );
        let merged =
            coalesce_temporal_vec(time_segments, Some(&AllowedIntervals::new([Interval::DAY])));
        assert_eq!(
            (merged[0].0.time_interval().seconds(), merged[0].0.t()),
            (1, [0, 3599])
        );
        // 秒区間そのものは変わらない。
        assert_eq!(merged[0].0.seconds_range(), (0, 3600));
    }

    /// FlexIdが違えば、時間が連続していても結合しない。
    #[test]
    fn does_not_merge_across_different_segments() {
        let a = SingleId::new(12, 0, 3638, 1614)
            .unwrap()
            .with_time(Interval::HOUR, 0)
            .unwrap();
        let b = SingleId::new(12, 0, 3639, 1614)
            .unwrap()
            .with_time(Interval::HOUR, 1)
            .unwrap();

        let mut time_segments: Vec<(FlexId, u8)> = Vec::new();
        time_segments.extend(a.into_iter().map(|id| (id, 7u8)));
        time_segments.extend(b.into_iter().map(|id| (id, 7u8)));

        assert_eq!(coalesce_temporal_vec(time_segments, None).len(), 2);
    }

    /// 時間を使っていない場合は入力と1対1で対応する（全時間Segmentはそのまま）。
    #[test]
    fn passes_through_when_no_temporal_information() {
        let time_segments: Vec<(FlexId, u8)> = (0..4u32)
            .map(|x| (FlexId::new(3, 0, 3, x, 3, 0).unwrap(), 1u8))
            .collect();

        let merged = coalesce_temporal_vec(time_segments, None);
        assert_eq!(merged.len(), 4);
        assert!(merged.iter().all(|(id, _)| id.is_whole_time()));
    }
}
