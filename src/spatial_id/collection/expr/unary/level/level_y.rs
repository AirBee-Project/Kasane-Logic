use super::{LevelParam, insert_leveled};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 南北（Y）方向の占有を絶対範囲 `[lo, hi]` へ揃える。Y方向は巡回せず範囲外はエラー。
pub struct YLevel;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for YLevel {
    type CustomParameter = LevelParam<u32, A>;
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
            for leveled in flex_id.level_y(z, lo, hi)? {
                insert_leveled(&mut result, leveled, value.clone(), &conflict);
            }
        }
        Ok(result)
    }
}
