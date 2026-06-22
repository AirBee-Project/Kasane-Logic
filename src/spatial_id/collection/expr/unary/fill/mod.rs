use crate::{
    CellValue, ConflictPolicy, Error, IterFlexIds, RangeId, SpatialIdCollection, UnaryOperator,
};

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
        // 既定値で bbox を埋めた後、実セルで上書きする。後勝ち（Overwrite）で実値が勝つよう、
        // 既定値 → 実セルの順に一括構築へ渡す。
        let bbox = RangeId::bounding_box_of(a.scan().map(|(flex_id, _)| flex_id));
        let defaults = bbox
            .as_ref()
            .into_iter()
            .flat_map(|b| b.iter_flex_ids())
            .map(move |flex_id| (flex_id, default.clone()));

        Ok(O::from_cells(
            defaults.chain(a.scan()),
            &ConflictPolicy::Overwrite,
        ))
    }

    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool {
        false
    }
}
