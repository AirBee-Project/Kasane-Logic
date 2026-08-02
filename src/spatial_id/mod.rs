pub mod collection;
pub mod single_id;
pub mod traits;
pub mod zoom_level;

/// 時間軸まわりの実装（`{i}` の型・候補集合・2進セル変換）。
pub mod time;

//非公開のモジュール
pub mod flex_id;
pub mod helpers;
pub mod range_id;

#[cfg(test)]
mod tests;
