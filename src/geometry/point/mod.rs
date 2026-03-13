use crate::Ecef;
pub mod coordinate;
pub mod ecef;

///現実空間の点に対して共通で定義することができる性質
pub trait Point: Into<Ecef> {}
