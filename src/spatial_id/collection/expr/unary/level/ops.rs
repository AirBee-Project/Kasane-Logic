use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::LevelParam;
use super::level_f::FLevel;
use super::level_x::XLevel;
use super::level_y::YLevel;

/// Level 系演算子を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// `stretch_*` が元の占有を相対量だけ継ぎ足して起伏を保存するのに対し、`level_*` は対象次元の
/// 占有を絶対座標範囲 `[lo, hi]` へ置き換える。元の位置は捨てられるため、その次元の起伏（凹凸）は
/// 平坦化され、範囲を越えた占有は削られ、足りない区間は埋められる。
///
/// 範囲へ揃える過程で同一の列の複数セルが重なるため、値が衝突したときの解決方針を指定できる。
/// 引数なしの `level_*` は後勝ち（[`ConflictPolicy::Overwrite`]）で、`level_*_with` で方針を選べる。
/// 値が1種の `Set`/`bool` ではどの方針でも結果は同じ。
///
/// ```ignore
/// let flat = table.level_f(25, 0, 10)?;                              // F を [0,10] に平坦化（後勝ち）
/// let flat = table.level_f_with(25, 0, 10, ConflictPolicy::Max)?;    // 大きい値を優先
/// ```
pub trait LevelOps: SpatialIdCollection {
    /// F方向の占有を絶対範囲 `[lo, hi]` へ揃える（衝突は後勝ち）。
    fn level_f(&self, z: u8, lo: i32, hi: i32) -> Result<Self, Error> {
        self.level_f_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    /// X方向の占有を絶対範囲（`lo` から東向きに `hi` まで）へ揃える（衝突は後勝ち）。
    fn level_x(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_x_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    /// Y方向の占有を絶対範囲 `[lo, hi]` へ揃える（範囲外はエラー・衝突は後勝ち）。
    fn level_y(&self, z: u8, lo: u32, hi: u32) -> Result<Self, Error> {
        self.level_y_with(z, lo, hi, ConflictPolicy::Overwrite)
    }

    /// F方向の占有を絶対範囲へ、衝突方針を指定して揃える。
    fn level_f_with(
        &self,
        z: u8,
        lo: i32,
        hi: i32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        FLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }

    /// X方向の占有を絶対範囲へ、衝突方針を指定して揃える。
    fn level_x_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        XLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }

    /// Y方向の占有を絶対範囲へ、衝突方針を指定して揃える。
    fn level_y_with(
        &self,
        z: u8,
        lo: u32,
        hi: u32,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        YLevel::execution::<Self, Self>(
            self,
            LevelParam {
                z,
                lo,
                hi,
                conflict,
            },
        )
    }
}

impl<C> LevelOps for C where C: SpatialIdCollection {}
