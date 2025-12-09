//! 時空間ID（Space ID）とエンコードされたIDを扱うモジュール
//!
//! このモジュールは、時空間座標をエンコード・デコードする機能と、
//! ID操作のための型を提供します。
//! 
//! # 主なモジュール
//! - [`space_id`]: 時空間IDの操作とトレイト定義
//! - [`encode`]: エンコードされたIDの表現

pub mod encode;
pub mod space_id;

pub use encode::EncodeID;
pub use space_id::SpaceId;
