// 時空間IDの集合を扱うための概念
pub mod collection;

// このライブラリが扱う3つの空間ID
pub mod flex_id;
pub mod range_id;
pub mod single_id;

// 時間ID
pub mod temporal_id;

// 共通定義の関数やTrait
pub mod helpers;
pub mod traits;

// ズームレベル型
pub mod zoom_level;

#[cfg(test)]
mod tests;
