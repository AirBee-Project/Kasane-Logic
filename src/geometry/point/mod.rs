use crate::Ecef;
pub mod coordinate;
pub mod ecef;

pub trait Point: Into<Ecef> {}
