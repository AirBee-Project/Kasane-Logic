use core::ops::Sub;

use crate::{ConflictPolicy, Error, SpatialIdCollection, UnaryOperator};

use super::DiffuseParam;
use super::diffuse_f::FDiffuse;
use super::diffuse_x::XDiffuse;
use super::diffuse_y::YDiffuse;

/// 値を持つセルから周囲へ、距離に応じて減衰させた値を波及させる演算。
///
/// 「建物（値を持つセル）の周りに段階的にリスクを設定する」といった用途を想定する。
/// 既定の衝突方針は [`ConflictPolicy::Max`]（重なったら高い方を残す）で、発生源の値は
/// そのまま保持される。方針を変えたい場合は `*_with` を使う。
pub trait DiffuseOps: SpatialIdCollection
where
    Self::Value: Sub<Output = Self::Value>,
{
    /// 高さ（F）方向へ波及させる（衝突は `Max`）。
    fn diffuse_f(&self, z: u8, distance: u32, decay: Self::Value) -> Result<Self, Error> {
        self.diffuse_f_with(z, distance, decay, ConflictPolicy::Max)
    }

    /// 東西（X）方向へ波及させる（巡回・衝突は `Max`）。
    fn diffuse_x(&self, z: u8, distance: u32, decay: Self::Value) -> Result<Self, Error> {
        self.diffuse_x_with(z, distance, decay, ConflictPolicy::Max)
    }

    /// 南北（Y）方向へ波及させる（境界はクリップ・衝突は `Max`）。
    fn diffuse_y(&self, z: u8, distance: u32, decay: Self::Value) -> Result<Self, Error> {
        self.diffuse_y_with(z, distance, decay, ConflictPolicy::Max)
    }

    /// 高さ（F）方向へ、衝突方針を指定して波及させる。
    fn diffuse_f_with(
        &self,
        z: u8,
        distance: u32,
        decay: Self::Value,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        FDiffuse::execution::<Self, Self>(
            self,
            DiffuseParam {
                z,
                distance,
                decay,
                conflict,
            },
        )
    }

    /// 東西（X）方向へ、衝突方針を指定して波及させる。
    fn diffuse_x_with(
        &self,
        z: u8,
        distance: u32,
        decay: Self::Value,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        XDiffuse::execution::<Self, Self>(
            self,
            DiffuseParam {
                z,
                distance,
                decay,
                conflict,
            },
        )
    }

    /// 南北（Y）方向へ、衝突方針を指定して波及させる。
    fn diffuse_y_with(
        &self,
        z: u8,
        distance: u32,
        decay: Self::Value,
        conflict: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        YDiffuse::execution::<Self, Self>(
            self,
            DiffuseParam {
                z,
                distance,
                decay,
                conflict,
            },
        )
    }
}

impl<C> DiffuseOps for C
where
    C: SpatialIdCollection,
    C::Value: Sub<Output = C::Value>,
{
}
