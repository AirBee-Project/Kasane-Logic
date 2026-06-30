use crate::spatial_id::collection::expr::query::unary::kernel::UnaryKernel;
use crate::spatial_id::collection::expr::unary::level::{Level, LevelParam};
use crate::spatial_id::collection::expr::unary::shift::{Shift, ShiftParam};
use crate::spatial_id::collection::expr::unary::spread::{Spread, SpreadParam};
use crate::spatial_id::collection::expr::unary::stretch::{Stretch, StretchParam};
use crate::{Error, SpatialIdCollection, UnaryOperator};

#[cfg(feature = "rayon")]
pub type DynUnaryKernel<C> = dyn UnaryKernel<C> + Send + Sync;
#[cfg(not(feature = "rayon"))]
pub type DynUnaryKernel<C> = dyn UnaryKernel<C>;

pub enum UnaryOp<C: SpatialIdCollection> {
    Shift(ShiftParam),
    Stretch(StretchParam<C::Value>),
    Level(LevelParam<C::Value>),
    Spread(SpreadParam<C::Value>),
    Fill(C::Value),
    Custom(alloc::boxed::Box<DynUnaryKernel<C>>),
}

impl<C: SpatialIdCollection> UnaryOp<C> {
    /// このノードが恒等変換（入力をそのまま返す）かどうか。
    pub fn is_identity(&self) -> bool {
        use crate::spatial_id::collection::expr::unary::fill::FillDefault;

        match self {
            UnaryOp::Shift(p) => <Shift as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Stretch(p) => <Stretch as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Level(p) => <Level as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Spread(p) => <Spread as UnaryOperator<C::Value>>::is_identity(p),
            UnaryOp::Fill(v) => <FillDefault as UnaryOperator<C::Value>>::is_identity(v),
            UnaryOp::Custom(kernel) => kernel.is_identity(),
        }
    }

    pub fn run(self, input: &C) -> Result<C, Error> {
        match self {
            UnaryOp::Shift(p) => Shift::execution::<C, C>(input, p),
            UnaryOp::Stretch(p) => Stretch::execution::<C, C>(input, p),
            UnaryOp::Level(p) => Level::execution::<C, C>(input, p),
            UnaryOp::Spread(p) => Spread::execution::<C, C>(input, p),
            UnaryOp::Fill(v) => {
                crate::spatial_id::collection::expr::unary::fill::FillDefault::execution::<C, C>(
                    input, v,
                )
            }
            UnaryOp::Custom(kernel) => kernel.run(input),
        }
    }
}
