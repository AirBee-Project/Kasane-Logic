use crate::{Error, SpatialIdTable, UnaryOperator};

/// 東西（X）方向への移動を行う。X方向は地球を周回するため巡回する。
pub struct XShift;

pub enum XShiftCustomParameter {
    /// インデックス値で指定
    Index { z: u8, index: i32 },
}

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for XShift {
    type CustomParameter = XShiftCustomParameter;
    type ResultValue = A;

    fn execution(
        a: &SpatialIdTable<A>,
        custom_parameter: Self::CustomParameter,
    ) -> Result<SpatialIdTable<Self::ResultValue>, Error> {
        match custom_parameter {
            XShiftCustomParameter::Index { z, index } => {
                let mut result = SpatialIdTable::new();
                for (flex_id, value) in a.iter() {
                    for shifted in flex_id.x_shift(z, index)? {
                        result.insert(shifted, value.clone());
                    }
                }
                Ok(result)
            }
        }
    }
}
