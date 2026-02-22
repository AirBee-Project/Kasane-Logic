// pub mod core; // 新規追加
// pub mod set;
// pub mod table;

///集合の中から関連のある空間IDを検索する
// pub mod scanner;

pub type FlexIdRank = u64;
pub type ValueRank = u64;

/// Rankを貯めておく個数
/// `FlexId`と`Value`で統一。
pub const RECYCLE_RANK_MAX: usize = 1024;
