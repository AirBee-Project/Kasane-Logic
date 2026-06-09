#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

/// 和集合
pub mod union;

/// 積集合
pub mod intersection;

/// 差集合
pub mod difference;

/// 対称差
pub mod symmetric_difference;

/// マスク（積・左値保持）
pub mod mask;

/// 異型の値合成（4演算の一般化）
pub mod combine;

/// 集合演算をメソッドとして呼び出す拡張トレイト
pub mod ops;

#[cfg(test)]
mod tests;
