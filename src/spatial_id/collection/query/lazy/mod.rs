use crate::{
    Error,
    spatial_id::collection::query::{execution::Query, traits::WorkingTree},
};

#[cfg(test)]
mod tests;

/// 遅延ビュー。
///
/// 対象領域から必要な入力領域を逆算し、その部分だけを入力源から読んで評価する。
/// 入力源に要求するのは [`Source::read_subset`](crate::Source::read_subset) だけ。
pub struct LazyView<'a, W: WorkingTree + 'static> {
    pub(crate) query: &'a Query<W>,
}

impl<'a, W: WorkingTree + 'static> LazyView<'a, W>
where
    W::Value: 'static,
{
    pub fn get<T: crate::SpatialId>(
        &self,
        target: T,
    ) -> Result<impl Iterator<Item = (crate::FlexId, W::Value)>, Error> {
        let req_bounds = alloc::vec![target.clone().into()];
        let working = self.query.run_on_subset(req_bounds)?;
        let target_range: crate::RangeId = target.into();

        Ok(working
            .into_iter()
            .filter(move |(id, _)| id.intersects_range(&target_range)))
    }

    /// 対象領域(`target`)のうち、データが存在しない空間を `default_value` で埋めてから値を返します。
    pub fn get_with_default<T: crate::SpatialId>(
        &self,
        target: T,
        default_value: W::Value,
    ) -> Result<impl Iterator<Item = (crate::FlexId, W::Value)>, Error> {
        let req_bounds = alloc::vec![target.clone().into()];
        let working = self.query.run_on_subset(req_bounds)?;
        let target_range: crate::RangeId = target.clone().into();

        let mut uncovered = crate::SpatialIdSet::new();
        uncovered.insert(target.into());
        let mut covered_results = alloc::vec::Vec::new();

        for (id, value) in working.into_iter() {
            if id.intersects_range(&target_range) {
                for _ in uncovered.remove(&id) {}
                covered_results.push((id, value));
            }
        }

        let default_results = uncovered
            .into_iter()
            .map(move |(id, _)| (id, default_value.clone()));

        Ok(covered_results.into_iter().chain(default_results))
    }
}
