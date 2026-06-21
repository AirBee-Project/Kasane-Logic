use crate::spatial_id::collection::expr::plan::unary::kernel::UnaryKernel;
use crate::spatial_id::collection::expr::unary::level::{Level, LevelParam};
use crate::spatial_id::collection::expr::unary::shift::{Shift, ShiftParam};
use crate::spatial_id::collection::expr::unary::stretch::{Stretch, StretchParam};
use crate::{Error, FusibleOperator, SpatialIdCollection, UnaryOperator};

pub enum UnaryOp<C: SpatialIdCollection> {
    /// F / X / Y のいずれか or 複数軸の移動。軸ごとの `shift_x` などはこの形で表現され、
    /// 連続する Shift は最適化で軸が衝突しない範囲へ融合される。
    Shift(ShiftParam),
    /// F / X / Y のいずれかの引き延ばし。軸ごとの `stretch_x` などはこの形で表現される。
    Stretch(StretchParam<C::Value>),
    /// F / X / Y のいずれかの絶対範囲揃え。軸ごとの `level_x` などはこの形で表現される。
    Level(LevelParam<C::Value>),
    Fill(C::Value),
    Custom(alloc::boxed::Box<dyn UnaryKernel<C>>),
}

impl<C: SpatialIdCollection> UnaryOp<C> {
    /// このノードが恒等変換（入力をそのまま返す）かどうか。
    pub fn is_identity(&self) -> bool {
        use crate::spatial_id::collection::expr::unary::fill::FillDefault;

        match self {
            UnaryOp::Shift(p) => <Shift as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Stretch(p) => <Stretch as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Level(p) => <Level as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Fill(v) => <FillDefault as UnaryOperator<C::Value>>::is_identity(v),
            UnaryOp::Custom(kernel) => kernel.is_identity(),
        }
    }

    pub fn run(self, input: &C) -> Result<C, Error> {
        match self {
            UnaryOp::Shift(p) => Shift::execution::<C, C>(input, p),
            UnaryOp::Stretch(p) => Stretch::execution::<C, C>(input, p),
            UnaryOp::Level(p) => Level::execution::<C, C>(input, p),
            UnaryOp::Fill(v) => {
                crate::spatial_id::collection::expr::unary::fill::FillDefault::execution::<C, C>(
                    input, v,
                )
            }
            UnaryOp::Custom(kernel) => kernel.run(input),
        }
    }

    /// 直後（内側）に適用される `inner` を自分へ融合できれば、融合した演算子を `Ok` で返す。
    /// 融合できなければ両演算子をそのまま `Err` で返し戻す。
    pub(crate) fn try_fuse(self, inner: Self) -> Result<Self, alloc::boxed::Box<(Self, Self)>> {
        match (self, inner) {
            (UnaryOp::Shift(outer), UnaryOp::Shift(inner)) => {
                match <Shift as FusibleOperator>::fuse(outer, inner) {
                    Ok(fused) => Ok(UnaryOp::Shift(fused)),
                    Err((outer, inner)) => Err(alloc::boxed::Box::new((
                        UnaryOp::Shift(outer),
                        UnaryOp::Shift(inner),
                    ))),
                }
            }
            (outer, inner) => Err(alloc::boxed::Box::new((outer, inner))),
        }
    }
}
