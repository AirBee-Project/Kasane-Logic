use crate::{Error, SpatialIdTable, UnaryOperator};

/// 南北（Y）方向への移動を行う。Y方向は巡回せず、範囲外への移動はエラーになる。
pub struct YShift;

pub enum YShiftCustomParameter {
    /// インデックス値で指定
    Index { z: u8, index: i32 },
}

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for YShift {
    type CustomParameter = YShiftCustomParameter;
    type ResultValue = A;

    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error> {
        match custom_parameter {
            YShiftCustomParameter::Index { z, index } => {
                let mut result = SpatialIdTable::new();
                for (flex_id, value) in a.iter() {
                    for shifted in flex_id.y_shift(z, index)? {
                        result.insert(shifted, value.clone());
                    }
                }

                Ok(result)
            }
        }
    }
}
