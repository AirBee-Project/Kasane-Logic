use crate::Ecef;

///現実空間の点に対して共通で定義することができる性質
pub trait Point: Into<Ecef> {}
