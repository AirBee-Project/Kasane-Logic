use super::ShiftParam;
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 高さ方向への移動を行う
pub struct FShift;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for FShift {
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
            for shifted in flex_id.shift_f(z, index)? {
                result.insert(shifted, value.clone());
            }
        }
        Ok(result)
    }

    fn is_identity(custom_parameter: &Self::CustomParameter) -> bool {
        custom_parameter.index == 0
    }
}
