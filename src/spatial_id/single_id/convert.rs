use crate::{FlexId, SingleId};

impl IntoIterator for SingleId {
    type Item = FlexId;
    type IntoIter = core::iter::Once<FlexId>;
    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(unsafe {
            FlexId::new_unchecked(self.z(), self.f(), self.z(), self.x(), self.z(), self.y())
        })
    }
}

impl SingleId {
    pub fn single_ids(self) -> impl Iterator<Item = SingleId> {
        core::iter::once(self)
    }
}
