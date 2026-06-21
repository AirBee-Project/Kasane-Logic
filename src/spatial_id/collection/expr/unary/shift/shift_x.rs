use super::ShiftParam;
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 東西（X）方向への移動を行う。X方向は地球を周回するため巡回する。
pub struct XShift;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for XShift {
    type CustomParameter = ShiftParam;
    type ResultValue = A;

    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let ShiftParam { z, index } = custom_parameter;

        let mut result = O::empty();
        for (flex_id, value) in a.scan() {
            for shifted in flex_id.shift_x(z, index)? {
                result.insert(shifted, value.clone());
            }
        }
        Ok(result)
    }

    fn is_identity(param: &Self::CustomParameter) -> bool {
        param.index == 0
    }
}
