//! 空間 ID の操作トレイトとその実装
//!
//! 本モジュールは、時空間 ID の基本操作を定義する [`SpaceID`] トレイトと、その実装である`SingleId`および`RangeId`を提供します。
//!
//! # 空間 ID の種類
//! 本ライブラリが扱う空間 ID は 2 種類です。いずれも IPA が定める標準的な空間 ID 形状に準拠しており、相互変換も可能です。ユースケースに応じてパフォーマンス特性が異なるため、両者を使い分けることを推奨します。
//!
//! [`SpaceID`] トレイトは、すべての空間 ID 型が満たすべき基礎的な性質を定義します。
//!
//! ## `SingleId`
//! `SingleId` は各次元が単一値で表現される標準的な空間 ID です。分散した位置を扱う場合や、単純なアルゴリズム設計で有用です。
//!
//! ```ignore
//! pub struct SingleId {
//!     z: u8,
//!     f: i64,
//!     x: u64,
//!     y: u64,
//! }
//! ```
//! ## `RangeId`
//! `RangeId` は各次元のインデックスを 2 つの値による区間で表現します。連続した広い範囲を一度に扱えるため、`SingleId` より高いパフォーマンスを発揮する場面があります。
//!
//! ```ignore
//! pub struct RangeId {
//!     z: u8,
//!     f: [i64; 2],
//!     x: [u64; 2],
//!     y: [u64; 2],
//! }
//! ```

use crate::{error::Error, geometry::coordinate::Coordinate};

//ユーザーに対して公開されているモジュール
pub mod constants;
pub mod range;
pub mod single;

//非公開のモジュール
pub(crate) mod encode;
pub(crate) mod helpers;

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpatialId {
    //そのIDの各次元の最大と最小を返す
    fn min_f(&self) -> i64;
    fn max_f(&self) -> i64;
    fn max_xy(&self) -> u64;

    //各インデックスの移動
    fn move_f(&mut self, by: i64) -> Result<(), Error>;
    fn move_x(&mut self, by: i64);
    fn move_y(&mut self, by: i64) -> Result<(), Error>;

    //中心点の座標を求める関数
    fn center(&self) -> Coordinate;

    //頂点をの座標を求める関数
    fn vertices(&self) -> [Coordinate; 8];

    //EncodeIdの集合に変換するメゾット
}
