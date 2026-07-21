use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{
    Error, FlexId,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};
use alloc::vec::Vec;

/// コレクションを包む最小の[crate::RangeId]内の空領域をデフォルト値で埋める単項演算子。
pub struct FillEmpty<V> {
    pub default_value: V,
}

impl<V> FillEmpty<V> {
    pub fn new(default_value: V) -> Self {
        Self { default_value }
    }
}

impl<W> UnaryOperator<W> for FillEmpty<W::Value>
where
    W: WorkingTree,
    W::Value: 'static,
{
    fn run(&self, core: &mut W) -> Result<(), Error> {
        let Some(bbox) = core.bounding_box() else {
            return Ok(());
        };

        let default_items: Vec<(FlexId, W::Value)> = bbox
            .into_iter()
            .map(|id| (id, self.default_value.clone()))
            .collect();

        let default_tree = W::from_flexids(default_items);
        *core = default_tree.overlay(core);

        Ok(())
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }

    fn validate(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn commutativity_info(&self) -> CommutativityInfo {
        CommutativityInfo::none()
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "fill_empty")
    }
}
