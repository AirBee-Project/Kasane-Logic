use crate::spatial_id::{
    collection::{Collection, Rank},
    flex_id::FlexId,
};

pub fn find_related<C: Collection>(collection: &C, target: FlexId) -> Vec<Rank> {
    todo!()
}
