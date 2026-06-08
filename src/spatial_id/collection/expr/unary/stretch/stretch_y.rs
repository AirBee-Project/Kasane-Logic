use super::{StretchParam, insert_stretched};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 南北（Y）方向への引き延ばしを行う。Y方向は巡回せず、範囲外への拡張はエラーになる。
pub struct YStretch;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for YStretch {
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
            for stretched in flex_id.stretch_y(z, index)? {
                insert_stretched(&mut result, stretched, value.clone(), &conflict);
            }
        }
        Ok(result)
    }
}
