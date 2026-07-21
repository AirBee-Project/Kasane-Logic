use crate::{
    Error, SpatialIdCollection,
    spatial_id::collection::query::{execution::Query, traits::WorkingTree},
};

#[cfg(test)]
mod tests;

/// 遅延評価ビュー。指定された空間に対する部分計算を提供する。
pub struct LazyView<'a, S: SpatialIdCollection> {
    pub(crate) query: &'a Query<S>,
}

impl<'a, S: SpatialIdCollection> LazyView<'a, S>
where
    S::Working: 'static,
    S::Value: 'static,
{
    /// 指定された `target` の空間のみを計算して結果を返す。
    /// `target` は `SpatialId` トレイトを実装する任意の型（`SingleId`, `FlexId`, `RangeId` 等）を受け付ける。
    /// その空間に内包・交差するすべての値を抽出して返す。
    pub fn get<T: crate::SpatialId>(
        &self,
        target: T,
    ) -> Result<alloc::vec::Vec<(crate::FlexId, S::Value)>, Error> {
        let req_bounds = alloc::vec![target.clone().into()];
        let _subset_bounds = self.query.inverse_bounds(req_bounds[0].clone());
        let working = self.query.run_on_subset(&req_bounds)?;
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
