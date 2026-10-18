use crate::spatial_id::collection::flex_tree::core::NoSummary;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::{
    merge_policy::MergePolicy,
    traits::{BinaryOperator, QueryIter},
};
use crate::{Error, SpatialIdTable};

/// `MergePolicy` で重ね合わせる二項演算子。
pub struct Merge<V, P> {
    default: V,
    _marker: core::marker::PhantomData<fn() -> P>,
}

impl<V, P> Merge<V, P> {
    pub fn new(default: V) -> Self {
        Self {
            default,
            _marker: core::marker::PhantomData,
        }
    }
}

impl<V: SafeValue + 'static, P> BinaryOperator<V> for Merge<V, P>
where
    P: MergePolicy<V>,
{
    /// 両側に値がある場所は `resolve(a, b)`、片側だけの場所は欠けた側を `default` として解決する。
    fn run<'a>(
        &self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error> {
        let mut target_a: SpatialIdTable<V, NoSummary> = lhs.collect::<Result<_, _>>()?;
        let target_b: SpatialIdTable<V, NoSummary> = rhs.collect::<Result<_, _>>()?;
        let (a, b) = (&target_a.inner, &target_b.inner);
        let resolve = |x: &V, y: &V| P::resolve(x.clone(), y.clone());

        let mut merged = a.intersection(b);
        for (id, value) in b.intersection(a) {
            merged.insert_with(id, value, resolve);
        }
        merged.extend(
            a.difference(b)
                .into_iter()
                .map(|(id, value)| (id, resolve(&value, &self.default))),
        );
        merged.extend(
            b.difference(a)
                .into_iter()
                .map(|(id, value)| (id, resolve(&self.default, &value))),
        );
        target_a.inner = merged;
        Ok(target_a.into())
    }

    fn reverse_target(
        &self,
        output_target: crate::RangeId,
    ) -> (Option<crate::RangeId>, Option<crate::RangeId>) {
        (Some(output_target.clone()), Some(output_target))
    }
}
