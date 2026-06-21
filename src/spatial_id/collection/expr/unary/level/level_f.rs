use super::{LevelParam, insert_leveled};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 高さ（F）方向の占有を絶対範囲 `[lo, hi]` へ揃える。F方向の起伏を平坦化する。
pub struct FLevel;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for FLevel {
    type CustomParameter = LevelParam<i32, A>;
    type ResultValue = A;

    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = A>,
    {
        let LevelParam {
            z,
            lo,
            hi,
            conflict,
        } = custom_parameter;

        let mut result = O::empty();
        for (flex_id, value) in a.scan() {
            for leveled in flex_id.level_f(z, lo, hi)? {
                insert_leveled(&mut result, leveled, value.clone(), &conflict);
            }
        }
        Ok(result)
    }

    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool {
        false
    }
}
