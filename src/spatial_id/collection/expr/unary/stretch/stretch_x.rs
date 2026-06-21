use super::{StretchParam, insert_stretched};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 東西（X）方向への引き延ばしを行う。X方向は地球を周回するため巡回する。
pub struct XStretch;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for XStretch {
    type CustomParameter = StretchParam<A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let StretchParam { z, index, conflict } = custom_parameter;

        let mut result = O::empty();
        for (flex_id, value) in a.scan() {
            for stretched in flex_id.stretch_x(z, index)? {
                insert_stretched(&mut result, stretched, value.clone(), &conflict);
            }
        }
        Ok(result)
    }

    fn is_identity(custom_parameter: &Self::CustomParameter) -> bool {
        custom_parameter.index == 0
    }
}
