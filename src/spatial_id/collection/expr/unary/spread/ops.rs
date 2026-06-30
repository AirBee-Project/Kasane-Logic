use crate::{ConflictPolicy, SpatialIdCollection};

use super::{SpreadAxes, SpreadParam};

use crate::spatial_id::collection::expr::query::Query;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    /// 対象軸・衝突方針を明示して伝播する（最も一般的な入口）。
    pub fn spread_axes_with(
        self,
        z: u8,
        radius: u32,
        axes: SpreadAxes,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Unary(
            crate::spatial_id::collection::expr::query::UnaryOp::Spread(SpreadParam::new(
                z, radius, axes, decay, conflict,
            )),
            alloc::boxed::Box::new(self),
        )
    }

    /// X / Y 平面へ同心円状に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread(self, z: u8, radius: u32, decay: fn(&C::Value, u32) -> Option<C::Value>) -> Self {
        self.spread_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、XY 平面への [`spread`](Self::spread)。
    pub fn spread_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::XY, decay, conflict)
    }

    /// X 軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_x(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_x_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、X 軸沿いの [`spread_x`](Self::spread_x)。
    pub fn spread_x_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::X, decay, conflict)
    }

    /// Y 軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_y(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_y_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、Y 軸沿いの [`spread_y`](Self::spread_y)。
    pub fn spread_y_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::Y, decay, conflict)
    }

    /// F（高さ）軸沿い（1D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_f(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_f_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、F 軸沿いの [`spread_f`](Self::spread_f)。
    pub fn spread_f_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::F, decay, conflict)
    }

    /// X / Y / F 全軸へ同心球状（3D）に伝播する（重なりは [`ConflictPolicy::Max`]）。
    pub fn spread_xyz(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
    ) -> Self {
        self.spread_xyz_with(z, radius, decay, ConflictPolicy::Max)
    }

    /// 衝突解決方針を指定できる、3D 球の [`spread_xyz`](Self::spread_xyz)。
    pub fn spread_xyz_with(
        self,
        z: u8,
        radius: u32,
        decay: fn(&C::Value, u32) -> Option<C::Value>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        self.spread_axes_with(z, radius, SpreadAxes::XYZ, decay, conflict)
    }
}
