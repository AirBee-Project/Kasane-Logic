use crate::{Error, SpatialIdTable, UnaryOperator};

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

    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error> {
        todo!()
    }
}
