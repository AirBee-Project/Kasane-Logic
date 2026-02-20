use crate::spatial_id::collection::core::CollectionCore;

#[derive(Clone, Debug, Default)]
pub struct SetOnMemory {
    core: CollectionCore<()>,
}
