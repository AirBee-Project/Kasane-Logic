use crate::{Error, SpatialIdTable, UnaryOperator};

/// 高さ方向への移動を行う
pub struct FShift;

pub enum FShiftCustomParameter {
    /// インデックス値で指定
    Index { z: u8, index: i32 },
}

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for FShift {
    type CustomParameter = FShiftCustomParameter;
    type ResultValue = A;

    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error> {
        match custom_parameter {
            FShiftCustomParameter::Index { z, index } => {
                let mut result = SpatialIdTable::new();
                for (flex_id, value) in a.iter() {
                    for shifted in flex_id.f_shift(z, index)? {
                        result.insert(shifted, value.clone());
                    }
                }
                Ok(result)
            }
        }
    }
}
