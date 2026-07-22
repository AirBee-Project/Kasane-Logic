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

        Ok(working.into_iter().filter(move |(id, _)| {
            crate::spatial_id::collection::query::execution::intersects_flex_range(
                id,
                &target_range,
            )
        }))
    }

    /// 対象領域(`target`)のうち、データが存在しない空間を `default_value` で埋めてから値を返します。
    pub fn get_with_default<T: crate::SpatialId>(
        &self,
        target: T,
        default_value: W::Value,
    ) -> Result<impl Iterator<Item = (crate::FlexId, W::Value)>, Error> {
        let req_bounds = alloc::vec![target.clone().into()];
        let working = self.query.run_on_subset(req_bounds)?;

        // target を default_value で埋めた WorkingTree を作成
        let target_items = target
            .clone()
            .into_iter()
            .map(|id| (id, default_value.clone()))
            .collect::<alloc::vec::Vec<_>>();
        let base_tree = W::from_flexids(target_items);

        let overlay_tree = base_tree.overlay(&working);

        let target_range: crate::RangeId = target.into();
        Ok(overlay_tree.into_iter().filter(move |(id, _)| {
            crate::spatial_id::collection::query::execution::intersects_flex_range(
                id,
                &target_range,
            )
        }))
    }
}
