/// 集合演算（和・積・差・対称差・マスク・値合成）
pub mod set;

/// 算術演算（加算ほか）
pub mod arith;
pub mod kernel;
pub mod op;

pub use kernel::*;
pub use op::*;
