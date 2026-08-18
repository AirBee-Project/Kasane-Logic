use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::grid::GridAxis;
use crate::spatial_id::collection::query::working::WorkingTree;
use crate::{Error, ZoomLevel, spatial_id::collection::query::traits::UnaryOperator};

/// 作業木全体を高さ（F）方向へ、ズームレベル `z` のインデックス値 `f` 個分だけ平行移動する単項演算。
pub struct ShiftF {
    z: ZoomLevel,
    f: i32,
}

impl ShiftF {
    /// ズーム `z` のインデックス値 `f` 個分の高さ移動を表す演算子を作る。
    pub fn new<T: Into<u8>>(z: T, f: i32) -> Result<Self, Error> {
        let z = ZoomLevel::new(z.into())?;
        Ok(Self { z, f })
    }
}

impl<V: SafeValue + 'static> UnaryOperator<V> for ShiftF {
    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::Separable { policy: None }
    }

    fn validate(&self) -> Result<(), Error> {
        let zl = ZoomLevel::new(self.z.get())?;
        zl.check_f(self.f)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn run(&self, target: &mut WorkingTree<V>) -> Result<(), Error> {
        let z = self.z.get();
        let index = self.f;
        if index == 0 {
            return Ok(());
        }

        let rebuilt = target.core().map_rebuild(|id, value| {
            let value = value.clone();
            Ok(id
                .shift_f(z, index)?
                .map(move |moved| (moved, value.clone())))
        })?;
        *target = WorkingTree::from_core(rebuilt);
        Ok(())
    }

    fn inverse_bounds(&self, bounds: crate::RangeId) -> Option<crate::RangeId> {
        let z = self.z.get();
        let target_z = z.max(bounds.z());
        let delta = (self.f as i64) * (1i64 << (target_z - z));

        bounds.f_edges_shift(target_z, -delta, -delta).unwrap()
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "shift_f(z={}, f={})", self.z.get(), self.f)
    }

    fn grid_zoom(&self) -> Option<crate::ZoomLevel> {
        Some(self.z)
    }

    #[allow(private_interfaces)]
    fn apply_to_grid(
        &self,
        grid: &mut crate::spatial_id::collection::query::grid::UniformGrid<V>,
    ) -> Result<crate::spatial_id::collection::query::grid::Applied, crate::Error> {
        grid.shift(GridAxis::F, self.z, self.f)
            .map(|_| crate::spatial_id::collection::query::grid::Applied::Done)
    }
}
