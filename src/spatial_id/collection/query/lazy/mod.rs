use crate::{
    Error, SpatialIdCollection,
    spatial_id::collection::query::{execution::Query, traits::WorkingTree},
};

#[cfg(test)]
mod tests;

/// 遅延ビュー
pub struct LazyView<'a, S: SpatialIdCollection> {
    pub(crate) query: &'a Query<S>,
}

impl<'a, S: SpatialIdCollection> LazyView<'a, S>
where
    S::Working: 'static,
    S::Value: 'static,
{
    pub fn get<T: crate::SpatialId>(
        &self,
        target: T,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, S::Value)>, Error> {
        let req_bounds = alloc::vec![target.clone().into()];
        let working = self.query.run_on_subset(req_bounds)?;
        let mut results = alloc::vec::Vec::new();
        let target_range: crate::RangeId = target.into();
        for (id, val) in working.iter_ref() {
            if crate::spatial_id::collection::query::execution::intersects_flex_range(
                &id,
                &target_range,
            ) {
                results.push((id, val.clone()));
            }
        }
        Ok(results)
    }
}
