use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::StretchParam;
use super::stretch_f::FStretch;
use super::stretch_x::XStretch;
use super::stretch_y::YStretch;

/// Stretch 系演算子を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 演算子型（`FStretch` 等）はそのまま残しつつ、`SpatialIdCollection` を実装する任意の
/// コレクション（`Table` / `Map` / `Set` …）へメソッドを生やす。`shift_*` がセルを移動するのに
/// 対し、`stretch_*` は元のセルを残したまま指定方向へ占有を拡張する。
///
/// 伸長は重なりを生むため、値が衝突したときの解決方針を指定できる。引数なしの
/// `stretch_*` は後勝ち（[`ConflictPolicy::Overwrite`]）で、`stretch_*_with` で方針を選べる。
/// 値が1種の `Set`/`bool` ではどの方針でも結果は同じ。
///
/// ```ignore
/// let grown = table.stretch_f(25, 3)?;                              // 後勝ち
/// let grown = table.stretch_f_with(25, 3, ConflictPolicy::Max)?;    // 大きい値を優先
/// ```
pub trait StretchOps: SpatialIdCollection {
    /// 高さ（F）方向へ引き延ばす（衝突は後勝ち）。
    fn stretch_f(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_f_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 東西（X）方向へ引き延ばす（巡回・衝突は後勝ち）。
    fn stretch_x(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_x_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 南北（Y）方向へ引き延ばす（範囲外はエラー・衝突は後勝ち）。
    fn stretch_y(&self, z: u8, index: i32) -> Result<Self, Error> {
        self.stretch_y_with(z, index, ConflictPolicy::Overwrite)
    }

    /// 高さ（F）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_f_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        FStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }

    /// 東西（X）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_x_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        XStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }

    /// 南北（Y）方向へ、衝突方針を指定して引き延ばす。
    fn stretch_y_with(
        &self,
        z: u8,
        index: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        YStretch::execution::<Self, Self>(self, StretchParam { z, index, conflict })
    }
}

impl<C> StretchOps for C where C: SpatialIdCollection {}
