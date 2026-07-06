use crate::{FlexId, IterFlexIds, IterSingleIds, SingleId};

impl IterFlexIds for SingleId {
    type Iter<'a> = core::iter::Once<FlexId>;
    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        core::iter::once(FlexId::from(self))
    }
}

impl IterSingleIds for SingleId {
    type Iter<'a> = core::iter::Once<SingleId>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        core::iter::once(self.clone())
    }
}
