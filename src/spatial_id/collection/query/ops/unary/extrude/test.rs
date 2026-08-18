use crate::ZoomLevel;
use crate::spatial_id::collection::query::merge_policy::Sum;
use crate::spatial_id::collection::query::traits::UnaryOperator;

use super::extrude_x::ExtrudeX;

/// bounds のx軸が折り返し（`x[0] > x[1]`）の場合でも、target の固定範囲との重なり判定が
/// 正しく行われることを確認する回帰テスト。
///
/// `x_fine_range` は折り返しを考慮せず min > max のまま返すため、単純な区間判定
/// （`target_max < bounds_min || bounds_max < target_min`）をそのまま使うと、
/// 実際には重なっているのに重ならないと誤判定してしまう。
///
/// bounds.x = [5, 2]（z=3、{5,6,7,0,1,2} を表す折り返し範囲）は target の [1,1] と
/// x=1 のところで重なるはずなので、`Some(_)` が返るべき。
#[test]
fn extrude_x_inverse_bounds_overlaps_wrapped_bounds() {
    let op = ExtrudeX::<Sum>::new(ZoomLevel::new(3).unwrap(), 1, 1);

    let mut bounds = crate::RangeId::new(3, 0, 0, 0).unwrap();
    bounds.set_x([5, 2]).unwrap();

    let inv = <ExtrudeX<Sum> as UnaryOperator<u32>>::inverse_bounds(&op, bounds);
    assert!(
        inv.is_some(),
        "折り返した bounds との重なりを見落としている"
    );
}
