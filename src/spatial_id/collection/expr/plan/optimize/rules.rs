use alloc::boxed::Box;

use crate::SpatialIdCollection;
use crate::spatial_id::collection::expr::plan::Plan;

use super::Rewrite;

/// 恒等な単項演算を取り除く。
pub(super) fn drop_identity_unary<C: SpatialIdCollection>(plan: Plan<C>) -> Rewrite<Plan<C>>
where
    C::Value: 'static,
{
    match plan {
        Plan::Unary(op, input) if op.is_identity() => Rewrite::Changed(*input),
        other => Rewrite::Unchanged(other),
    }
}

/// 隣接する 2 つの単項演算を、可能なら 1 つへ融合する。
///
/// 融合できるか・どう融合するかは [`UnaryOp::try_fuse`](super::super::UnaryOp::try_fuse) に委ねる。
/// 1 回の適用では隣接ペアのみを見るが、[`apply_until_stable`](super::Plan::apply_until_stable)
/// が繰り返すため連続する Shift も順に畳まれる。
pub(super) fn fuse_adjacent_unary<C: SpatialIdCollection>(plan: Plan<C>) -> Rewrite<Plan<C>>
where
    C::Value: 'static,
{
    match plan {
        Plan::Unary(op, input) => match *input {
            // 単項演算が 2 段重なっているときだけ融合を試みる。
            Plan::Unary(inner, rest) => match op.try_fuse(inner) {
                Ok(fused) => Rewrite::Changed(Plan::Unary(fused, rest)),
                Err(pair) => {
                    let (op, inner) = *pair;
                    Rewrite::Unchanged(Plan::Unary(op, Box::new(Plan::Unary(inner, rest))))
                }
            },
            // 子が単項でなければそのまま戻す。
            other => Rewrite::Unchanged(Plan::Unary(op, Box::new(other))),
        },
        other => Rewrite::Unchanged(other),
    }
}

/// 可換な二項演算の被演算子順序を正規化し、重い部分木を左へ寄せる。
pub(super) fn canonicalize_commutative<C: SpatialIdCollection>(plan: Plan<C>) -> Rewrite<Plan<C>>
where
    C::Value: 'static,
{
    match plan {
        Plan::Binary(op, lhs, rhs)
            if op.is_commutative() && rhs.node_count() > lhs.node_count() =>
        {
            Rewrite::Changed(Plan::Binary(op, rhs, lhs))
        }
        other => Rewrite::Unchanged(other),
    }
}
