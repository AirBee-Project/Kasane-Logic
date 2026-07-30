pub mod collection;
pub mod single_id;
pub mod traits;
pub mod zoom_level;

/// 時間間隔 `{i}`。公開 API に出てくる唯一の時間の型。
pub mod interval;

//非公開のモジュール
pub mod flex_id;
pub mod helpers;
pub mod range_id;

/// 絶対秒区間と2進セルの相互変換（クレート内部専用）。
pub(crate) mod time_cells;

#[cfg(test)]
mod tests;
