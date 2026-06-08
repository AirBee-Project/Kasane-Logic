use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::ShiftParam;
use super::shift_f::FShift;
use super::shift_x::XShift;
use super::shift_y::YShift;

/// Shift 系演算子を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 演算子型（`FShift` 等）はクエリ言語の AST 用にそのまま残しつつ、`SpatialIdCollection`
/// を実装する任意のコレクション（`Table` / `Map` / `Set` …）へメソッドを生やす。
///
/// ```ignore
/// let moved = table.shift_f(25, 3)?; // FShift::execution の糖衣
/// let moved = set.shift_x(25, 5)?;   // Set でもそのまま使える
/// ```
pub trait ShiftOps: SpatialIdCollection {
    /// 高さ（F）方向へ、ズーム `z` のセル `index` 個分だけ移動する。
    fn shift_f(&self, z: u8, index: i32) -> Result<Self, Error> {
        FShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }

    /// 東西（X）方向へ、ズーム `z` のセル `index` 個分だけ移動する（巡回する）。
    fn shift_x(&self, z: u8, index: i32) -> Result<Self, Error> {
        XShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }

    /// 南北（Y）方向へ、ズーム `z` のセル `index` 個分だけ移動する（範囲外はエラー）。
    fn shift_y(&self, z: u8, index: i32) -> Result<Self, Error> {
        YShift::execution::<Self, Self>(self, ShiftParam { z, index })
    }
}

impl<C> ShiftOps for C where C: SpatialIdCollection {}
