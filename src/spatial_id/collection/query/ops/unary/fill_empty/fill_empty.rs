use crate::{
    Error, FlexId,
    spatial_id::collection::query::traits::{UnaryOperator, WorkingTree},
};
use alloc::vec::Vec;

/// コレクションを包む包摂境界（`RangeId`）内の空領域をデフォルト値で埋める単項演算子。
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
{
    fn run(&self, core: &mut W) -> Result<(), Error> {
        let Some(bbox) = core.bounding_box() else {
            return Ok(());
        };

        let default_items: Vec<(FlexId, W::Value)> = bbox
            .into_iter()
            .map(|id| (id, self.default_value.clone()))
            .collect();

        let default_tree = W::from_items(default_items);
        *core = default_tree.overlay(core);

        Ok(())
    }
}
