#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::Coordinate;
pub mod geometry_relation;
pub mod impls;
#[cfg(test)]
mod tests;

///直線を表す型
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub points: [Coordinate; 2],
}

impl Line {
    ///[Line]を作成する。
    pub fn new(points: [Coordinate; 2]) -> Self {
        Self { points }
    }
}
