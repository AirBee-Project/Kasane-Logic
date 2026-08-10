pub mod falloff_f;
pub mod falloff_x;
pub mod falloff_y;
pub mod primitive;
pub mod query;

#[cfg(test)]
mod test;

use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::grid::{GridAxis, GridOp};
use crate::spatial_id::collection::query::merge_policy::MergePolicy;
use crate::spatial_id::helpers::Side;
use crate::spatial_id::zoom_level::ZoomLevel;
use alloc::boxed::Box;
use core::convert::TryFrom;
use core::fmt::Debug;
use core::ops::{Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FalloffPattern {
    Linear,
    QuadraticIn,
    QuadraticOut,
}

/// F/X/Y の 3 演算子で共通の [`grid_op`](crate::spatial_id::collection::query::traits::UnaryOperator::grid_op)。
///
/// 軸以外は完全に同じなので、3 つの実装に同じ本体を書き写さず、軸を引数で受ける。
///
/// グリッドは畳み込みの順序を保証しない（レーン走査と整列後の畳み込みで順が変わる）ので、
/// 順序に結果が依存しない可換なポリシーのときだけ `Some` を返す。
pub(crate) fn grid_op<V, P>(
    axis: GridAxis,
    z: ZoomLevel,
    radius: u32,
    direction: Option<Side>,
    pattern: FalloffPattern,
) -> Option<GridOp<V>>
where
    V: SafeValue + Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32>,
    <V as TryFrom<u32>>::Error: Debug,
    P: MergePolicy<V> + Send + Sync + 'static,
{
    if !P::IS_COMMUTATIVE {
        return None;
    }
    Some(GridOp::falloff(
        axis,
        z,
        radius,
        direction,
        Box::new(move |value: &V, distance: u32| attenuate(value, distance, radius, pattern)),
        Box::new(|a, b| P::resolve(a.clone(), b.clone())),
    ))
}

/// 線形減衰の定義そのもの。距離 `distance` にある点へ `value` がどこまで届くかを返す。
///
/// `distance == 0` で元の値、`distance == radius` で 0 になる線形補間。
///
/// この演算は 2 つの経路で実行される。木を降りて 2r+1 倍に展開する経路
/// （[`FlexId::falloff_x`](crate::FlexId::falloff_x) 等）と、一様ズームの
/// 平坦表現をレーン走査する経路
/// （[`UnaryOperator::grid_op`](crate::spatial_id::collection::query::traits::UnaryOperator::grid_op)）。
/// 走査の仕方は別物で構わないが、**減衰の定義が 2 か所にあると、片方だけ直したときに
/// 経路によって結果が変わる**。そこで定義はここ 1 か所に置き、両経路がこれを呼ぶ。
///
/// `radius == 0` では呼ばないこと（ゼロ除算になる）。半径 0 は恒等変換として
/// 呼び出し側が先に弾いている。
pub(crate) fn attenuate<V>(value: &V, distance: u32, radius: u32, pattern: FalloffPattern) -> V
where
    V: Mul<Output = V> + Div<Output = V> + Sub<Output = V> + TryFrom<u32> + Clone,
    <V as TryFrom<u32>>::Error: Debug,
{
    let v_distance = V::try_from(distance).unwrap();
    let v_radius = V::try_from(radius).unwrap();
    match pattern {
        FalloffPattern::Linear => {
            (value.clone() * (v_radius - v_distance)) / V::try_from(radius).unwrap()
        }
        FalloffPattern::QuadraticIn => {
            // 1 - (d/r)^2 = (r^2 - d^2) / r^2
            let r_sq = v_radius.clone() * v_radius.clone();
            let d_sq = v_distance.clone() * v_distance;
            (value.clone() * (r_sq.clone() - d_sq)) / r_sq
        }
        FalloffPattern::QuadraticOut => {
            // (1 - d/r)^2 = (r - d)^2 / r^2
            let r_sq = v_radius.clone() * v_radius.clone();
            let r_minus_d = v_radius - v_distance;
            let diff_sq = r_minus_d.clone() * r_minus_d;
            (value.clone() * diff_sq) / r_sq
        }
    }
}
