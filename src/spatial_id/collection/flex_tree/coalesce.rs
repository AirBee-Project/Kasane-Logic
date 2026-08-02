//! 木から読み出したセル列を、時間方向に結合してから書き出すための層。
//!
//! FlexTree は時間軸を「2の冪秒の2進セル」として持つため、`Interval` が2の冪でない時間 ID
//! （仕様が認める `1800` 秒など）は挿入時に複数セルへ分解される。読み出しでセルを1つずつ
//! そのまま返すと、`12/0/3638/1614_1800/809712` が5つの断片になってしまい、
//! 仕様書の `{i}/{t}` 表記が失われる。
//!
//! ここでは「空間セルと値が同じで、時間が隣接している」セルどうしを結合してから
//! `RangeId::with_time_span` に渡す。結合後の秒区間は元の区間に戻るので、
//! `gcd` ベースの復元によって `1800/809712` がそのまま取り出せる。
//!
//! 結合は [`FlexId`] では表現できない（[`FlexId`] は2進セル1個しか持てない）ため、
//! 出力は [`RangeId`] である。したがってこの層は「[`RangeId`] / [`SingleId`] を書き出す経路」
//! （`flat_single_ids` と JSON 直列化）専用で、`get` / `iter` のような [`FlexId`] を返す
//! 経路は従来どおり生のセルを返す。

use alloc::vec::Vec;

use crate::{FlexId, IntervalSet, RangeId};

/// 時間方向に隣接するセルを結合し、[`RangeId`] の列として返す。
///
/// `units` に[`IntervalSet`]を渡すと、結合後の秒区間を**その候補のうち最も粗い単位**で
/// 表し直す（セル数が候補の中で最小になる）。`None` の場合は「その区間を表せる最も粗い単位」
/// （`gcd(開始秒, 幅)`）になり、セル数は常に1になる。
///
/// [`IntervalSet`] は必ず全区間を表せる候補を含むので、**この関数は失敗しない**。
///
/// # 計算量
///
/// 入力を `(空間セル, 開始秒)` でソートしてから1回走査するので `O(n log n)`。
/// 時間を使っていない木（全セルが `WHOLE`）でも結合対象が見つからないだけで、
/// 結果は入力と1対1に対応する。
pub(crate) fn coalesce_temporal<V>(
    items: impl IntoIterator<Item = (FlexId, V)>,
    units: Option<&IntervalSet>,
) -> Vec<(RangeId, V)>
where
    V: Clone + PartialEq,
{
    // (空間だけのFlexId, 開始秒, 終了秒, 値) へ落とす。
    // 空間キーは時間成分を取り除いた FlexId そのもの（`Ord` を持つのでソートに使える）。
    let mut rows: Vec<(FlexId, u64, u64, V)> = items
        .into_iter()
        .map(|(flex_id, value)| {
            let (start, end) = flex_id.seconds_range();
            (spatial_key(&flex_id), start, end, value)
        })
        .collect();

    if rows.is_empty() {
        return Vec::new();
    }

    // 同じ空間セルのものを固め、その中で時間順に並べる。
    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    let mut out: Vec<(RangeId, V)> = Vec::with_capacity(rows.len());

    let mut iter = rows.into_iter();
    let (mut key, mut start, mut end, mut value) = iter.next().expect("空でないことは確認済み");

    for (next_key, next_start, next_end, next_value) in iter {
        // 空間セルと値が同じで、時間が隙間なく続いているなら伸ばす。
        if next_key == key && next_value == value && next_start == end {
            end = next_end;
            continue;
        }
        out.push(finish(&key, start, end, value, units));
        key = next_key;
        start = next_start;
        end = next_end;
        value = next_value;
    }
    out.push(finish(&key, start, end, value, units));

    out
}

/// 時間成分を全時間へ落とした [`FlexId`]。空間だけを見たグルーピングのキーに使う。
fn spatial_key(flex_id: &FlexId) -> FlexId {
    flex_id.with_time_cell(0, 0)
}

/// 結合し終えた1件を [`RangeId`] に組み立てる。
fn finish<V>(
    key: &FlexId,
    start: u64,
    end: u64,
    value: V,
    units: Option<&IntervalSet>,
) -> (RangeId, V) {
    let range = RangeId::from(key)
        .with_time_span(start, end)
        .expect("結合した秒区間は元のセルの和なので常に有効");

    // 候補集合が指定されていれば、その中で最も粗い（＝セル数が最小の）単位へ表し直す。
    // `IntervalSet` は必ず全区間を表せる候補を含むので、この `relabel_time` は失敗しない。
    let range = match units {
        None => range,
        Some(units) => range
            .clone()
            .relabel_time(units.coarsest_dividing(start, end))
            .expect("IntervalSet が選んだ単位は必ずこの区間を割り切る"),
    };

    (range, value)
}

#[cfg(all(test, feature = "temporal_id"))]
mod tests {
    use super::*;
    use crate::{Interval, SingleId, SpatialId};

    /// 仕様書 1.5.3 の例（`1800/809712`）が、分解 → 結合を経て元の表記へ戻る。
    #[test]
    fn round_trips_the_spec_example() {
        let original = SingleId::new(12, 0, 3638, 1614)
            .unwrap()
            .with_time(1800, 809712)
            .unwrap();

        // 木へ入れるときと同じ分解を通す。
        let cells: Vec<_> = original.clone().into_iter().map(|id| (id, 1u8)).collect();
        assert!(cells.len() > 1, "1800秒は複数の2進セルへ分解されるはず");

        let merged = coalesce_temporal(cells, None);
        assert_eq!(merged.len(), 1, "結合されて1件になるはず");
        assert_eq!(merged[0].0.to_string(), "12/0/3638/1614_1800/809712");
    }

    /// 値が違うセルは結合しない。
    #[test]
    fn does_not_merge_across_different_values() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut cells: Vec<(FlexId, u8)> = Vec::new();
        cells.extend(a.into_iter().map(|id| (id, 1u8)));
        cells.extend(b.into_iter().map(|id| (id, 2u8)));

        let merged = coalesce_temporal(cells, None);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].0.seconds_range(), (0, 3600));
        assert_eq!(merged[1].0.seconds_range(), (3600, 7200));
    }

    /// 同じ値なら、隣接する2つの時間セルは1つに融合する。
    #[test]
    fn merges_adjacent_cells_with_the_same_value() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut cells: Vec<(FlexId, u8)> = Vec::new();
        cells.extend(a.into_iter().map(|id| (id, 7u8)));
        cells.extend(b.into_iter().map(|id| (id, 7u8)));

        let merged = coalesce_temporal(cells, None);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].0.seconds_range(), (0, 7200));
        // gcd(0, 7200) = 7200 なので「2時間」という単位で1セルになる。
        assert_eq!(
            (merged[0].0.interval().seconds(), merged[0].0.t()),
            (7200, [0, 0])
        );
    }

    /// 候補集合を渡すと、その中で最も粗い（＝セル数が最小の）単位が選ばれる。
    #[test]
    fn candidate_set_picks_the_coarsest_unit() {
        let base = SingleId::new(12, 0, 3638, 1614).unwrap();
        let a = base.clone().with_time(Interval::HOUR, 0).unwrap();
        let b = base.with_time(Interval::HOUR, 1).unwrap();

        let mut cells: Vec<(FlexId, u8)> = Vec::new();
        cells.extend(a.into_iter().map(|id| (id, 7u8)));
        cells.extend(b.into_iter().map(|id| (id, 7u8)));

        let merged = coalesce_temporal(cells, Some(IntervalSet::calendar()));
        assert_eq!(merged.len(), 1);
        assert_eq!(
            (merged[0].0.interval().seconds(), merged[0].0.t()),
            (3600, [0, 1])
        );

        // 候補が区間を割り切れないときは、必ず含まれる SECOND まで落ちる。
        // 候補集合が貧しいとセル数が爆発することを示す例でもある
        // （1時間 = 3600 セル）。実用では `IntervalSet::calendar()` のように
        // 粒度の階段を用意しておくこと。
        let mut cells: Vec<(FlexId, u8)> = Vec::new();
        cells.extend(
            SingleId::new(12, 0, 3638, 1614)
                .unwrap()
                .with_time(Interval::HOUR, 0)
                .unwrap()
                .into_iter()
                .map(|id| (id, 7u8)),
        );
        let merged = coalesce_temporal(cells, Some(&IntervalSet::new([Interval::DAY])));
        assert_eq!(
            (merged[0].0.interval().seconds(), merged[0].0.t()),
            (1, [0, 3599])
        );
        // 秒区間そのものは変わらない。
        assert_eq!(merged[0].0.seconds_range(), (0, 3600));
    }

    /// 空間セルが違えば、時間が連続していても結合しない。
    #[test]
    fn does_not_merge_across_different_cells() {
        let a = SingleId::new(12, 0, 3638, 1614)
            .unwrap()
            .with_time(Interval::HOUR, 0)
            .unwrap();
        let b = SingleId::new(12, 0, 3639, 1614)
            .unwrap()
            .with_time(Interval::HOUR, 1)
            .unwrap();

        let mut cells: Vec<(FlexId, u8)> = Vec::new();
        cells.extend(a.into_iter().map(|id| (id, 7u8)));
        cells.extend(b.into_iter().map(|id| (id, 7u8)));

        assert_eq!(coalesce_temporal(cells, None).len(), 2);
    }

    /// 時間を使っていない場合は入力と1対1で対応する（全時間セルはそのまま）。
    #[test]
    fn passes_through_when_no_temporal_information() {
        let cells: Vec<(FlexId, u8)> = (0..4u32)
            .map(|x| (FlexId::new(3, 0, 3, x, 3, 0).unwrap(), 1u8))
            .collect();

        let merged = coalesce_temporal(cells, None);
        assert_eq!(merged.len(), 4);
        assert!(merged.iter().all(|(id, _)| id.is_whole_time()));
    }
}
