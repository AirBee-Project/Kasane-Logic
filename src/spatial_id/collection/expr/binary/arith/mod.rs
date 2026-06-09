#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

/// 加算
pub mod add;

/// 減算
pub mod sub;

/// 乗算
pub mod mul;

pub mod ops;

#[cfg(test)]
mod tests;
