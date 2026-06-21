use crate::{CellValue, Error, IterFlexIds, RangeId, SpatialIdCollection, UnaryOperator};

pub mod ops;

#[cfg(test)]
mod tests;

/// 値を持つ領域を包む最小範囲の中で、まだ値が無いセルへ既定値を割り当てる演算。
pub struct FillDefault;

impl<A: CellValue> UnaryOperator<A> for FillDefault {
    /// 隙間へ割り当てる既定値。
    type CustomParameter = A;
    type ResultValue = A;

    fn execution<S, O>(a: &S, default: A) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let mut result = O::empty();
        if let Some(bbox) = RangeId::bounding_box_of(a.scan().map(|(flex_id, _)| flex_id)) {
            for flex_id in bbox.iter_flex_ids() {
                result.insert(flex_id, default.clone());
            }
        }

        for (flex_id, value) in a.scan() {
            result.insert(flex_id, value);
        }

        Ok(result)
    }

    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool {
        false
    }
}
