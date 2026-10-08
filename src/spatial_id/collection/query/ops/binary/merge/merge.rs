use crate::spatial_id::collection::flex_tree::core::{NoSummary, SafeValue};
use crate::spatial_id::collection::query::{merge_policy::MergePolicy, traits::BinaryOperator};
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

impl<V: SafeValue, P> BinaryOperator<V> for Merge<V, P>
where
    P: MergePolicy<V>,
{
    /// 両側に値がある場所は `resolve(a, b)`、片側だけの場所は欠けた側を `default` として解決する。
    fn run(
        &self,
        target_a: &mut SpatialIdTable<V, NoSummary>,
        target_b: &SpatialIdTable<V, NoSummary>,
    ) -> Result<(), Error> {
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
        Ok(())
    }

    fn inverse_bounds(
        &self,
        output_bounds: crate::RangeId,
    ) -> (Option<crate::RangeId>, Option<crate::RangeId>) {
        (Some(output_bounds.clone()), Some(output_bounds))
    }

    fn fmt_op(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "merge({})", P::NAME)
    }
}
