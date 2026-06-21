use super::{LevelParam, insert_leveled};
use crate::{Error, SpatialIdCollection, UnaryOperator};

/// 東西（X）方向の占有を絶対範囲へ揃える。X方向は巡回するため `lo` から東向きに `hi` まで。
pub struct XLevel;

impl<A: Ord + PartialEq + Clone> UnaryOperator<A> for XLevel {
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
            for leveled in flex_id.level_x(z, lo, hi)? {
                insert_leveled(&mut result, leveled, value.clone(), &conflict);
            }
        }
        Ok(result)
    }

    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool {
        false
    }
}
