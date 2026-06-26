use crate::{FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, SingleId};

impl IntoFlexIds for SingleId {
    type IntoIter = core::iter::Once<FlexId>;
    fn into_flex_ids(self) -> Self::IntoIter {
        core::iter::once(
            FlexId::new(self.z(), self.f(), self.z(), self.x(), self.z(), self.y()).unwrap(),
        )
    }
}

impl IterFlexIds for SingleId {
    type Iter<'a> = core::iter::Once<FlexId>;
    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.clone().into_flex_ids()
    }
}

impl IntoSingleIds for SingleId {
    type IntoIter = core::iter::Once<SingleId>;
    fn into_single_ids(self) -> Self::IntoIter {
        core::iter::once(self)
    }
}

impl IterSingleIds for SingleId {
    type Iter<'a> = core::iter::Once<SingleId>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        self.clone().into_single_ids()
    }
}
