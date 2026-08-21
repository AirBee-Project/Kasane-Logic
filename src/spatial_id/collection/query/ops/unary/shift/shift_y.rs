use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::grid::GridAxis;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, ZoomLevel, spatial_id::collection::query::traits::UnaryOperator};

/// 作業木全体を南北（Y）方向へ、ズームレベル `z` のインデックス値 `y` 個分だけ平行移動する単項演算。
pub struct ShiftY {
    z: ZoomLevel,
    y: i32,
}

impl ShiftY {
    /// ズーム `z` のインデックス値 `y` 個分の南北移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, y: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, y })
    }
}

impl<V: SafeValue + 'static> UnaryOperator<V> for ShiftY {
    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_y(self.y.unsigned_abs())?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error> {
        let z = self.z.get();
        let index = self.y;
        if index == 0 {
            return Ok(());
        }

        let rebuilt = target.core().map_rebuild(|id, value| {
            let value = value.clone();
            Ok(id.shift_y(z, index)?.map(move |m| (m, value.clone())))
        })?;
        *target = WorkingTree::from_core(rebuilt);
        Ok(())
    }

    fn forward_map(
        &self,
        id: crate::FlexId,
        value: V,
        out: &mut alloc::vec::Vec<(crate::FlexId, V)>,
    ) -> Result<(), crate::Error> {
        if self.y == 0 {
            out.push((id, value));
            return Ok(());
        }
        for new_id in id.shift_y(self.z.get(), self.y)? {
            out.push((new_id, value.clone()));
        }
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.y as i64) * (1i64 << (target_z - z));

        bounds.y_edges_shift(target_z, -delta, -delta).unwrap()
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::Separable { policy: None }
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_y(z={}, y={})", self.z.get(), self.y)
    }

    fn grid_zoom(&self) -> Option<crate::ZoomLevel> {
        Some(self.z)
    }

    #[allow(private_interfaces)]
    fn apply_to_grid(
        &self,
        grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
        token: &crate::CancellationToken,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, crate::Error> {
        grid.shift(GridAxis::Y, self.z, self.y, token)
            .map(|_| crate::spatial_id::collection::query::grid::Applied::Done)
    }
}
