//! 空間 ID の操作トレイトとその実装
//!
//! 本モジュールは、時空間 ID の基本操作を定義する [`SpaceID`] トレイトと、その実装である`SingleID`および`RangeID`を提供します。
//!
//! # 空間 ID の種類
//! 本ライブラリが扱う空間 ID は 2 種類です。いずれも IPA が定める標準的な空間 ID 形状に準拠しており、相互変換も可能です。ユースケースに応じてパフォーマンス特性が異なるため、両者を使い分けることを推奨します。
//!
//! [`SpaceID`] トレイトは、すべての空間 ID 型が満たすべき基礎的な性質を定義します。
//!
//! ## `SingleID`
//! `SingleID` は各次元が単一値で表現される標準的な空間 ID です。分散した位置を扱う場合や、単純なアルゴリズム設計で有用です。
//!
//! ```ignore
//! pub struct SingleID {
//!     z: u8,
//!     f: i64,
//!     x: u64,
//!     y: u64,
//! }
//! ```
//! ## `RangeID`
//! `RangeID` は各次元のインデックスを 2 つの値による区間で表現します。連続した広い範囲を一度に扱えるため、`SingleID` より高いパフォーマンスを発揮する場面があります。
//!
//! ```ignore
//! pub struct RangeID {
//!     z: u8,
//!     f: [i64; 2],
//!     x: [u64; 2],
//!     y: [u64; 2],
//! }
//! ```
//!
//! 各次元の範囲には順序が意味を持ち、通常は `[α, β]` の形で `α <= β` を想定します。ただし `α >= β` の場合は区間が反転し、空間の循環方向を表します。例えば Xインデックス ではWEB メルカトル法に基づく経度の循環性のため、これは実空間上の連続性と一致します。
//!
//! 一方 Yインデックス は WEB メルカトル法の制約（高緯度の非対応）により、反転区間は必ずしも実空間上の連続性を意味しません。Fインデックス については XYインデックス との対称性を考慮し境界循環を定義していますが、`α >= β` の場合は実空間的な連続性を保証しません。
//!
//!
//! 各構造体の詳細仕様やメソッドについては、それぞれの型のドキュメントを参照してください。
//!
//! # 移動操作
//! [`SpaceID`] トレイトは空間 ID の移動および基本情報取得のメソッドを定義します。
//! 特に移動メソッドには2種類存在します。多くの都市規模のユースケースでは `bound_*` の利用が安全と考えられますが、地球規模のデータ処理では `wrap_*` が適切な場面も存在します。利用側で方針が異なるため、本ライブラリは両方の操作を提供しています。
//!
//!## * **`bound_*` 系メソッド**  
//!   WEB メルカトル法や高度の上限に達した場合、境界を越えずエラーを返します。
//!
//!## * **`wrap_*` 系メソッド**  
//!   境界で循環します。Xインデックスでは経度の循環性に対応します。、YインデックスではWEBメルカトル法で循環しているように見えますが、実世界ではWEB メルカトル法の制約（高緯度の非対応）により連続でないことに注意してください。Fインデックスについても XYインデックス の次元対称性のため循環を定義していますが実空間的な連続性を保証しません。
//!

use crate::{error::Error, geometry::point::coordinate::Coordinate};

//ユーザーに対して公開されているモジュール
pub mod constants;
pub mod range;
pub mod single;

//非公開のモジュール
pub(crate) mod encode;
pub(crate) mod helpers;
pub(crate) mod segment;

/// 空間 ID が備えるべき基礎的な性質および移動操作を定義するトレイト。
pub trait SpaceID {
    //そのIDの各次元の最大と最小を返す
    fn min_f(&self) -> i64;
    fn max_f(&self) -> i64;
    fn max_xy(&self) -> u64;

    //WEBメルカトル法や高度の上限に来るとエラーを出す

    //基礎的な方角への移動
    fn bound_up(&mut self, by: u64) -> Result<(), Error>;
    fn bound_down(&mut self, by: u64) -> Result<(), Error>;
    fn bound_north(&mut self, by: u64) -> Result<(), Error>;
    fn bound_south(&mut self, by: u64) -> Result<(), Error>;
    fn bound_east(&mut self, by: u64) -> Result<(), Error>;
    fn bound_west(&mut self, by: u64) -> Result<(), Error>;
    //各インデックスの移動
    fn bound_f(&mut self, by: i64) -> Result<(), Error>;
    fn bound_x(&mut self, by: i64) -> Result<(), Error>;
    fn bound_y(&mut self, by: i64) -> Result<(), Error>;

    //WEBメルカトル法や高度の上限に来ると反対側に循環する

    //基礎的な方角への移動
    fn wrap_up(&mut self, by: u64);
    fn wrap_down(&mut self, by: u64);
    fn wrap_north(&mut self, by: u64);
    fn wrap_south(&mut self, by: u64);
    fn wrap_east(&mut self, by: u64);
    fn wrap_west(&mut self, by: u64);
    //各インデックスの移動
    fn wrap_f(&mut self, by: i64);
    fn wrap_x(&mut self, by: i64);
    fn wrap_y(&mut self, by: i64);

    //中心点の座標を求める関数
    fn center(&self) -> Coordinate;

    //頂点をの座標を求める関数
    fn vertices(&self) -> [Coordinate; 8];
}
