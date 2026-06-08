use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 高さ方向の変形を行う
pub struct FExtent;

/// パラメーター
pub enum FExtentCustomParameter {
    // 倍率変化
    // 最大値と最小値を決めた変化
}

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for FExtent {
    type CustomParameter = FExtentCustomParameter;
    type ResultValue = A;

    fn execution<S, O>(_a: &S, _custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        todo!()
    }
}
