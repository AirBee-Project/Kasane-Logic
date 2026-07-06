//! `from_cells`（結果組み立ての共通経路）の検証。
//!
//! Overwrite 経路は完全一致セルを畳み込み・最後の出現順で積む最適化をするため、
//! 完全重複・クロスズーム重なりを混ぜても、素朴な逐次 insert と同じ結果になることを確認する。

#[cfg(test)]
use crate::{ConflictPolicy, FlexId, SpatialIdCollection, SpatialIdTable};

#[cfg(test)]
fn sorted(table: &SpatialIdTable<i32>) -> alloc::vec::Vec<(FlexId, i32)> {
    let mut v: alloc::vec::Vec<(FlexId, i32)> = table.iter().map(|(id, val)| (id, *val)).collect();
    v.sort();
    v
}

#[cfg(test)]
fn reference(cells: &[(FlexId, i32)]) -> SpatialIdTable<i32> {
    // 逐次 insert（後勝ち）をグラウンドトゥルースとする。
    let mut table = SpatialIdTable::new();
    for (id, value) in cells {
        table.insert(id.clone(), *value);
    }
    table
}

#[test]
fn from_cells_overwrite_matches_sequential_insert() {
    let fine = FlexId::new(5, 2, 5, 2, 5, 2).unwrap(); // z5 の細かいセル
    let coarse = FlexId::new(4, 1, 4, 1, 4, 1).unwrap(); // fine を含む粗いセル

    // 完全重複・粗→細・細→粗・再上書きを織り交ぜる。
    let cells: alloc::vec::Vec<(FlexId, i32)> = alloc::vec![
        (fine.clone(), 1),
        (fine.clone(), 2),   // 完全重複 → 後勝ち
        (coarse.clone(), 3), // 粗いセルが fine を覆う
        (fine.clone(), 4),   // 後から細かいセルで一部上書き
        (coarse.clone(), 5), // さらに粗いセルで全面上書き（fine も消える）
        (fine.clone(), 6),   // 最後にまた細かいセル
    ];

    let bulk: SpatialIdTable<i32> =
        SpatialIdCollection::from_cells(cells.clone(), &ConflictPolicy::Overwrite);

    assert_eq!(sorted(&bulk), sorted(&reference(&cells)));
}

#[test]
fn from_cells_overwrite_many_exact_duplicates() {
    // 同一セルを大量に積む（起伏平坦化の縮約に相当）。最後の値だけが残る。
    let id = FlexId::new(20, 3, 20, 7, 20, 9).unwrap();
    let cells: alloc::vec::Vec<(FlexId, i32)> = (0..1000).map(|i| (id.clone(), i)).collect();

    let bulk: SpatialIdTable<i32> =
        SpatialIdCollection::from_cells(cells.clone(), &ConflictPolicy::Overwrite);

    assert_eq!(sorted(&bulk), alloc::vec![(id, 999)]);
}
