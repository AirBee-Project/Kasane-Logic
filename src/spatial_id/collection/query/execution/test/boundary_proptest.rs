//! 境界条件（F/Yの範囲端、Xの経度180度またぎ）に特化したプロパティテスト。
//!
//! `proptest_query.rs` の乱数テストは z=10..=12・オフセット ±5 程度に留まっており、
//! 範囲端やX方向の周回を偶然踏む確率はほぼゼロ。AST最適化（可換な演算の並べ替え）は
//! ちょうどこうした境界で意味が変わりうる
//! （`grid` モジュールのドキュメントが説明する「F/Yのfalloffは葉単位で全か無か」という
//! 挙動を参照）。ここでは意図的に境界へ置いたSegmentへ、可換な演算
//! （shift_f/shift_x/shift_y/falloff_linear_x）を宣言順を入れ替えて適用し、
//! `raw_run`（宣言順のまま）・`run`（最適化後に並べ替え）・`run_on_subset`
//! （`lazy_get` の実体。`&self` のまま `plan_order` で並べ替える経路）が常に一致する
//! ことを確認する。
//!
//! ズームを 3〜5 と浅くしているのは、範囲端（F/Y）やXの周長にすぐ到達できるようにする
//! ため。木が小さいのでケース数を増やしても十分速い（既定で `#[ignore]` しない）。

use crate::spatial_id::collection::query::merge_policy::Max;
use crate::{RangeId, SingleId, Source, SpatialIdTable, ZoomLevel};
use alloc::vec::Vec;
use proptest::prelude::*;

/// `(z, f, x, y, df, dx, dy, radius, falloff_first)` をまとめて生成する。
///
/// F/Y/Xの範囲端値はズーム `z` が決まらないと計算できないので `prop_flat_map` で
/// 段階的に組み立てる。オフセットは各演算子の `validate()` が要求する範囲
/// （`check_f`/`check_x`/`check_y`）に収める。範囲を超えると `run()` は
/// `validate()` で先に弾かれるが `raw_run()` は素通りするため、そこでの食い違いは
/// 最適化のバグではなく無関係な偽陽性になってしまう。
fn arb_boundary_case() -> impl Strategy<Value = (u8, i32, u32, u32, i32, i32, i32, u32, bool)> {
    (3..=5u8).prop_flat_map(|z| {
        let zl = ZoomLevel::new(z).unwrap();
        let f_min = zl.f_min();
        let f_max = zl.f_max();
        let xy_max = zl.xy_max();

        let f_strategy = prop_oneof![Just(f_min), Just(0), Just(f_max)];
        let x_strategy = prop_oneof![Just(0u32), Just(xy_max / 2), Just(xy_max)];
        let y_strategy = prop_oneof![Just(0u32), Just(xy_max / 2), Just(xy_max)];
        let df_strategy = f_min..=f_max;
        let dx_strategy = -(xy_max as i32)..=(xy_max as i32);
        let dy_strategy = -(xy_max as i32)..=(xy_max as i32);
        let radius_strategy = 1..=3u32;

        (
            Just(z),
            f_strategy,
            x_strategy,
            y_strategy,
            df_strategy,
            dx_strategy,
            dy_strategy,
            radius_strategy,
            any::<bool>(),
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        .. ProptestConfig::default()
    })]

    /// F/Y範囲端・X周回付近のSegmentに shift_f/shift_x/shift_y/falloff_linear_x/y/f を
    /// 宣言順を入れ替えて適用しても、3経路すべてで結果が一致する。
    ///
    /// 同じ軸のshiftとfalloffは、以前は [`CommutativityInfo`] 上「互いに可換」として
    /// 並べ替え対象になっていたが、shiftは範囲外に出ると即座に `Err` を返す（境界で止まる）
    /// のに対しfalloffは範囲外にはみ出す寄与を距離ごとに黙って捨てる（`grid` モジュールの
    /// ドキュメント参照）という非対称性があり、並べ替えるとエラーの有無が変わってしまう
    /// バグがあった（[`same_axis_shift_and_falloff_are_not_reordered`] 参照）。修正済み。
    #[test]
    fn boundary_shift_and_falloff_agree_across_execution_paths(
        (z, f, x, y, df, dx, dy, radius, falloff_first) in arb_boundary_case()
    ) {
        let zl = ZoomLevel::new(z).unwrap();
        let mut table = SpatialIdTable::<i32>::new();
        if let Ok(id) = SingleId::new(z, f, x, y) {
            table.insert(id, 7);
        }
        if table.is_empty() {
            return Ok(());
        }

        // F/Y方向のfalloffは、伝播先が範囲外へはみ出す葉を「全か無か」で落とす
        // （`grid` モジュールのドキュメント参照）。X方向とは違う経路なので併せて踏む。
        let build = |t: SpatialIdTable<i32>| {
            let q = t.query();
            if falloff_first {
                q.falloff_linear_f(z, radius, Max)
                    .falloff_linear_x(z, radius, Max)
                    .falloff_linear_y(z, radius, Max)
                    .shift_f(z, df)
                    .shift_x(z, dx)
                    .shift_y(z, dy)
            } else {
                q.shift_f(z, df)
                    .shift_x(z, dx)
                    .shift_y(z, dy)
                    .falloff_linear_f(z, radius, Max)
                    .falloff_linear_x(z, radius, Max)
                    .falloff_linear_y(z, radius, Max)
            }
        };

        let raw: Result<SpatialIdTable<i32>, _> = build(table.clone()).raw_run_table();
        let run: Result<SpatialIdTable<i32>, _> = build(table.clone()).run_table();

        match (&raw, &run) {
            (Ok(a), Ok(b)) => assert_eq!(
                a.flat_single_ids().collect::<Vec<_>>(),
                b.flat_single_ids().collect::<Vec<_>>(),
                "raw_run と run で結果が食い違う \
                 (z={z}, f={f}, x={x}, y={y}, df={df}, dx={dx}, dy={dy}, r={radius}, falloff_first={falloff_first})"
            ),
            (Err(_), Err(_)) => {}
            _ => panic!(
                "raw_run/run でエラーの有無が食い違う: raw_ok={} run_ok={} \
                 (z={z}, f={f}, x={x}, y={y}, df={df}, dx={dx}, dy={dy}, r={radius}, falloff_first={falloff_first})",
                raw.is_ok(),
                run.is_ok()
            ),
        }

        // 遅延経路（lazy_get = run_on_subset）も、エラーにならないケースでは同じ結果になること。
        // 対象を全空間にすれば raw_run の全件と一致するはず。
        if let Ok(expected) = &raw {
            let lazy_query = build(table.clone());
            let bbox = RangeId::new(z, [zl.f_min(), zl.f_max()], [0, zl.xy_max()], [0, zl.xy_max()])
                .unwrap();
            if let Ok(iter) = lazy_query.lazy_get(bbox) {
                let mut got: Vec<_> = iter.collect();
                got.sort_unstable();

                let mut exp: Vec<_> = expected.iter().map(|(id, &v)| (id, v)).collect();
                exp.sort_unstable();

                assert_eq!(
                    got, exp,
                    "lazy_get が raw_run と食い違う \
                     (z={z}, f={f}, x={x}, y={y}, df={df}, dx={dx}, dy={dy}, r={radius}, falloff_first={falloff_first})"
                );
            }
        }
    }
}

/// [`boundary_shift_and_falloff_agree_across_execution_paths`] が見つけたバグの最小再現。
///
/// z=3（xy_max=7）で y=7（Yの最大値）に置いたSegmentに対して、
/// `falloff_linear_y(r=2)` → `shift_y(-7)` の順（宣言順）で適用すると：
/// 1. falloffがまず y∈{5,6,7} へ値を広げる（y<0側ははみ出すので黙って捨てられる）
/// 2. その後のshift_y(-7)が y=5,6,7 を y=-2,-1,0 へ動かそうとし、
///    y=-2 と y=-1 が範囲外なので `Err(YOutOfRange)` になる
///
/// もし最適化（[`Query::optimize`]、`expansion_ratio` の昇順で並べ替える）が
/// shiftをfalloffより先に動かしてしまうと、shift_y(-7) を**元の** y=7 だけに適用してから
/// （y=0、範囲内）falloffを広げることになり `Ok` になってしまう——`raw_run`（宣言順）
/// とエラーの有無が食い違う。
///
/// `CommutativityInfo`（[`super::super::group_commutative::types`]）は以前、shiftと
/// falloffを同じ軸でも「互いに可換」として扱っていた。しかしshiftは範囲外で即`Err`に
/// なるのに対しfalloffは範囲外の寄与を距離ごとに黙って捨てるという非対称性があるため、
/// 両者の宣言順を入れ替えるとエラーになるかどうかが変わってしまっていた
/// （このプロパティテストを書く過程で発見。`run_on_subset`/`plan_order` とは無関係で、
/// `group_commutative_ops`/`sort_commutative_ops` 自体の既存バグだった）。
/// 修正: 同じ軸で片方でも単射（shiftなど、範囲外で即エラー）なら不可換とし、
/// 異なる軸同士・同じ軸でも集約操作（falloffなど）同士は従来どおり可換のままにした。
#[test]
fn same_axis_shift_and_falloff_are_not_reordered() {
    let mut table = SpatialIdTable::<i32>::new();
    table.insert(SingleId::new(3, 0, 0, 7).unwrap(), 7);

    let build = |t: SpatialIdTable<i32>| t.query().falloff_linear_y(3, 2u32, Max).shift_y(3, -7);

    let raw: Result<SpatialIdTable<i32>, _> = build(table.clone()).raw_run_table();
    let run: Result<SpatialIdTable<i32>, _> = build(table.clone()).run_table();

    assert_eq!(
        raw.is_ok(),
        run.is_ok(),
        "raw_run（宣言順）と run（最適化後）でエラーの有無が食い違う: raw={raw:?} run={run:?}"
    );
    // 修正前はここで run が Ok になっていた（shiftが先に並べ替わっていたため）。
    // 同じ軸のshiftとfalloffが不可換になった今は、両方とも宣言順のまま
    // Err(YOutOfRange) になるはず。
    assert!(
        raw.is_err(),
        "この境界配置ではそもそも両方ともErrになるはず"
    );
}
