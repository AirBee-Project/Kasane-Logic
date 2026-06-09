use crate::{Error, IterFlexIds, RangeId, SpatialIdCollection, UnaryOperator};

/// fill 系演算子をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;

/// 値を持つ領域を包む最小範囲（F/X/Y の3次元AABB）の中で、まだ値が無いセルへ既定値を割り当てる演算。
///
/// もともと値があったセルはその値を保持し（既定値で上書きしない）、AABB の外側へは何も書き込まない。
/// 結果として AABB は隙間なく埋まり、既存セルは元の値・隙間は既定値となる。
///
/// [`CustomParameter`](UnaryOperator::CustomParameter) に隙間へ割り当てる既定値をとる。
pub struct FillDefault;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for FillDefault {
    /// 隙間へ割り当てる既定値。
    type CustomParameter = A;
    type ResultValue = A;

    fn execution<S, O>(a: &S, default: A) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let mut result = O::empty();

        // 値を持つ領域の AABB を既定値で塗る。`iter_flex_ids` は範囲を最小セル集合へ分解するため、
        // 体積に比例せず効率よく敷き詰められる。空集合では AABB が無いので何もしない。
        if let Some(bbox) = RangeId::bounding_box_of(a.scan().map(|(flex_id, _)| flex_id)) {
            for flex_id in bbox.iter_flex_ids() {
                result.insert(flex_id, default.clone());
            }
        }

        // 元の値を上書き挿入する。既定値を塗った上から載せ直すので、元セルは元の値へ戻る。
        for (flex_id, value) in a.scan() {
            result.insert(flex_id, value);
        }

        Ok(result)
    }
}
