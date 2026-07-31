//! シャード初期化（[`SpatialIdSet::new_in_shard`]）の挙動テスト。
//!
//! - 領域外への挿入は無視される
//! - 領域をまたぐ粗い挿入は領域へ切り詰められる
//! - 領域が交わらない集合同士の演算は早期に確定する

#![cfg(test)]

#[cfg(test)]
use alloc::vec::Vec;

use crate::{FlexId, SingleId, SpatialIdSet};

/// ズーム `z` の単一セルを表す [`FlexId`] 領域を作る。
fn region(z: u8, f: i32, x: u32, y: u32) -> FlexId {
    FlexId::new(z, f, z, x, z, y).unwrap()
}

#[test]
fn insert_inside_region_is_kept() {
    // ズーム2のタイル (0,0,0) をシャード領域にする。
    let mut set = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    // 領域内のより細かいセル。
    let inside = SingleId::new(4, 0, 1, 1).unwrap();
    set.insert(inside.clone());

    assert_eq!(set.count(), 1);
    // 取り出した結果が入力と一致する（切り詰めなし）。
    let got: Vec<FlexId> = set.iter().collect();
    assert_eq!(got, inside.into_iter().collect::<Vec<_>>());
}

#[test]
fn insert_outside_region_is_ignored() {
    let mut set = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    // 同じズーム2の別タイル → 領域外。
    set.insert(SingleId::new(2, 0, 3, 3).unwrap());

    assert!(set.is_empty());
    assert_eq!(set.count(), 0);
}

#[test]
fn coarse_insert_is_clipped_to_region() {
    let shard = region(2, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard);
    // ズーム0（全空間）を挿入 → 領域に切り詰められるはず。
    set.insert(SingleId::new(0, 0, 0, 0).unwrap());

    let got: Vec<FlexId> = set.iter().collect();
    assert_eq!(got, vec![shard]);
}

#[test]
fn intersection_of_disjoint_shards_is_empty() {
    let mut a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());

    let mut b = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    b.insert(SingleId::new(4, 0, 13, 13).unwrap());

    // 領域が交わらない → 交差は空（早期確定）。
    let inter = &a & &b;
    assert!(inter.is_empty());
}

#[test]
fn difference_of_disjoint_shards_is_lhs() {
    let mut a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());

    let mut b = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    b.insert(SingleId::new(4, 0, 13, 13).unwrap());

    // 領域が交わらない → 差は lhs そのまま（早期確定）。
    let diff = &a - &b;
    assert_eq!(diff, a);
}

#[test]
fn same_region_intersection_matches_overlap() {
    let shard = region(2, 0, 0, 0);
    let mut a = SpatialIdSet::new_in_shard(shard);
    a.insert(SingleId::new(4, 0, 1, 1).unwrap());
    a.insert(SingleId::new(4, 0, 2, 2).unwrap());

    let mut b = SpatialIdSet::new_in_shard(shard);
    b.insert(SingleId::new(4, 0, 2, 2).unwrap());

    let inter = &a & &b;
    let got: Vec<FlexId> = inter.iter().collect();
    assert_eq!(
        got,
        SingleId::new(4, 0, 2, 2)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>()
    );
}

#[test]
fn split_shard_then_merge_shards_roundtrips() {
    // Map と対称な split_shard / merge_shards が、分割→統合で元の集合に戻ることを確認。
    let shard = region(2, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard);
    set.insert(SingleId::new(4, 0, 1, 1).unwrap());
    set.insert(SingleId::new(4, 0, 2, 2).unwrap());
    set.insert(SingleId::new(4, 0, 3, 0).unwrap());

    let ((lr, lower), (ur, upper)) = set.split_shard().expect("sharded set must split");

    // 2子のシャード領域は親に内包される（被覆分割の不変条件）。
    assert_eq!(shard.intersection(&lr).as_ref(), Some(&lr));
    assert_eq!(shard.intersection(&ur).as_ref(), Some(&ur));

    // 統合すると領域・内容ともに元へ戻る。
    let merged = SpatialIdSet::merge_shards(shard, vec![lower, upper]).unwrap();
    assert_eq!(merged.shard(), Some(&shard));
    assert_eq!(merged, set);
}

#[test]
fn merge_shards_rejects_region_outside_parent() {
    // 親領域に内包されない子を渡すと InvalidShardMerge。
    let parent = region(2, 0, 0, 0);
    let outside = SpatialIdSet::new_in_shard(region(2, 0, 3, 3));
    assert!(SpatialIdSet::merge_shards(parent, vec![outside]).is_err());
}

#[test]
fn merge_shards_rejects_overlapping_children() {
    // 親には内包されるが、子同士が重なる（同一領域）→ 互いに素でないので拒否。
    let parent = region(1, 0, 0, 0);
    let a = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    let b = SpatialIdSet::new_in_shard(region(2, 0, 0, 0));
    assert!(SpatialIdSet::merge_shards(parent, vec![a, b]).is_err());
}

#[test]
fn merge_shards_rejects_shardless_child() {
    // シャード領域未設定（new()）の子は検証不能なので拒否。
    let parent = region(1, 0, 0, 0);
    let shardless = SpatialIdSet::new();
    assert!(SpatialIdSet::merge_shards(parent, vec![shardless]).is_err());
}

/// 全軸ズーム0のシャード領域（＝全空間）はレベル0（F軸）に居るので、普通に分割できる。
///
/// 分割レベルの算出を「覆っていない最初のレベル」で行うと、全空間はどのレベルでも
/// 覆っているため打ち切りが必要になり、分割不能になってしまっていた。
#[test]
fn splitting_a_whole_space_shard_works() {
    let whole = FlexId::new(0, 0, 0, 0, 0, 0).unwrap();
    let mut set = SpatialIdSet::new_in_shard(whole);
    set.insert(SingleId::new(3, 1, 1, 1).unwrap());

    let ((lower_region, _), (upper_region, _)) =
        set.split_shard().expect("全空間シャードも分割できる");
    // レベル0の軸はFなので、F方向に2分される。
    assert_eq!(lower_region.f_zoomlevel(), 1);
    assert_eq!(upper_region.f_zoomlevel(), 1);
    assert_eq!(lower_region.x_zoomlevel(), 0);
    assert_eq!(lower_region.y_zoomlevel(), 0);
}

/// シャード分割は軸をローテーションする。
///
/// 1軸に偏るとシャードがその軸方向の薄いスライスに退化し、地理データ（Fが薄くX/Yが広い）
/// では負荷が極端に偏る。分割レベルを「覆っていない最初のレベル」で求めていた頃は、
/// Fが1段でも深いと常にレベル0＝F軸が選ばれ、F方向にしか割れなくなっていた。
#[test]
fn split_shard_rotates_axes() {
    let mut set = SpatialIdSet::new_in_shard(FlexId::new(1, 0, 1, 0, 1, 0).unwrap());
    for f in 0..4 {
        for x in 0..4u32 {
            for y in 0..4u32 {
                set.insert(SingleId::new(6, f, x, y).unwrap());
            }
        }
    }

    for step in 0..8 {
        let Some(((region, lower), _)) = set.split_shard() else {
            panic!("step {step}: 分割できるはずの領域で None");
        };
        let (f, x, y) = (
            region.f_zoomlevel(),
            region.x_zoomlevel(),
            region.y_zoomlevel(),
        );
        assert!(
            f.max(x).max(y) - f.min(x).min(y) <= 1,
            "step {step}: 空間軸の分割が偏った (f/x/y = {f}/{x}/{y})"
        );
        set = lower;
    }
}

/// シャード領域と同じ粗さの葉は、分割時に各半分へ**クリップ**されなければならない。
///
/// `prune_path` が「`region` は木の中に実在するノードである」と仮定していた頃は、
/// 領域より粗い葉に当たると何もせず素通りしていた。その結果、同じ葉が両方のシャードへ
/// 未クリップのまま複製され、`merge_shards` で二重計上になっていた。
#[test]
fn a_leaf_as_coarse_as_the_region_is_clipped_into_each_half() {
    let shard = region(2, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard);
    // 領域ちょうど1セル。分割するとどちらの半分よりも粗い。
    set.insert(SingleId::new(2, 0, 0, 0).unwrap());

    let ((lower_region, lower), (upper_region, upper)) = set.split_shard().expect("分割できるはず");

    // それぞれ自分の領域の内側に収まっていること（はみ出していない）。
    for (r, piece) in [(lower_region, &lower), (upper_region, &upper)] {
        for id in piece.iter() {
            assert_eq!(
                id.intersection(&r),
                Some(id),
                "シャード {r} が領域外の {id} を保持している"
            );
        }
        piece.inner.assert_canonical();
    }

    // 統合すると元へ戻る（複製されていれば戻らない）。
    let merged = SpatialIdSet::merge_shards(shard, alloc::vec![lower, upper]).unwrap();
    assert_eq!(merged, set);
}

/// 木に時間方向の構造があるときは、T軸もシャードの分割対象になる。
///
/// T軸の領域は木の中にノードとして実在しないことがある（木は「対象がその軸を覆っている」
/// レベルを実体化しない）ため、`prune_path` がそれを扱えることの回帰テストでもある。
#[cfg(feature = "temporal_id")]
#[test]
fn split_shard_uses_the_temporal_axis_when_the_tree_has_time() {
    use crate::Interval;

    let shard = region(2, 0, 0, 0);
    let quarter = Interval::new(1u64 << 33).unwrap(); // 全時間の1/4
    let mut set = SpatialIdSet::new_in_shard(shard);
    // 全時間を埋めないので木にT軸の分岐が残る。
    for t in [0u64, 2] {
        set.insert(
            SingleId::new(4, 0, 1, 1)
                .unwrap()
                .with_time(quarter, t)
                .unwrap(),
        );
    }
    assert!(set.inner.has_temporal_split());

    // 4軸巡回なので、4回以内に必ずT軸の番が来る。
    let mut current = set.clone();
    let mut saw_temporal_split = false;
    for _ in 0..4 {
        let parent_region = *current.shard().unwrap();
        let Some(((lower_region, lower), (upper_region, upper))) = current.split_shard() else {
            break;
        };

        if lower_region.t_zoomlevel() > parent_region.t_zoomlevel() {
            saw_temporal_split = true;
            // T軸で割ったので、2つの領域は時間だけが違う。
            assert_eq!(lower_region.f_zoomlevel(), parent_region.f_zoomlevel());
            assert_eq!(upper_region.t(), lower_region.t() + 1);
        }

        for (r, piece) in [(lower_region, &lower), (upper_region, &upper)] {
            for id in piece.iter() {
                assert_eq!(
                    id.intersection(&r),
                    Some(id),
                    "シャード {r} が領域外の {id} を保持している"
                );
            }
            piece.inner.assert_canonical();
        }

        let merged =
            SpatialIdSet::merge_shards(parent_region, alloc::vec![lower.clone(), upper]).unwrap();
        assert_eq!(merged, current, "分割→統合で元に戻らない");

        current = lower;
    }

    assert!(saw_temporal_split, "4段以内にT軸の分割が現れなかった");
}

/// 時間方向の構造が無い木では、T軸を選んではならない。
///
/// 全ての葉が全時間のときT軸で割ると「前半に全部」「後半に全部」の2枚になり、どちらも
/// 元と同じ葉数を持つ。`should_split_shard` は `count()` で判定するので、これを選ぶと
/// シャーディングが収束しなくなる。
#[test]
fn split_shard_avoids_the_temporal_axis_without_time_structure() {
    let shard = region(1, 0, 0, 0);
    let mut set = SpatialIdSet::new_in_shard(shard);
    for x in 0..4u32 {
        for y in 0..4u32 {
            set.insert(SingleId::new(5, 1, x, y).unwrap());
        }
    }
    assert!(!set.inner.has_temporal_split());

    let mut current = set;
    for step in 0..8 {
        let Some(((lower_region, lower), _)) = current.split_shard() else {
            break;
        };
        assert_eq!(
            lower_region.t_zoomlevel(),
            0,
            "step {step}: 時間構造の無い木でT軸が選ばれた"
        );
        current = lower;
    }
}
