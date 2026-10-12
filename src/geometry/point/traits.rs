use crate::{Ecef, Error, RangeId, SingleId};

///現実空間の点に対して共通で定義することができる性質
pub trait Point: Into<Ecef> {
    fn single_id(&self, z: impl Into<u8>) -> Result<SingleId, Error>;
    fn range_id(&self, z: impl Into<u8>) -> Result<RangeId, Error> {
        Ok(RangeId::from(self.single_id(z)?))
    }
}
