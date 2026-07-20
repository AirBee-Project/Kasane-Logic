use crate::{
    Error,
    spatial_id::collection::query::{
        merge_policy::MergePolicy,
        traits::{BinaryOperator, WorkingTree},
    },
};

/// 2つの作業木を `MergePolicy` で重ね合わせる二項演算子。
///
/// 片側にしか値が無いセルは、もう片方を `default` とみなして `MergePolicy::resolve` に渡す
/// （例: `resolve(default, b)`）。両側とも値の無いセルは演算を行わず空のままにする。
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

impl<W, P> BinaryOperator<W> for Merge<W::Value, P>
where
    W: WorkingTree,
    P: MergePolicy<W::Value>,
{
    fn run(&self, target_a: &mut W, target_b: &W) -> Result<(), Error> {
        if target_a.count() == 0 && target_b.count() == 0 {
            return Ok(());
        }
        *target_a = target_a.merge_with_default(target_b, &self.default, |a, b| {
            P::resolve(a.clone(), b.clone())
        });
        Ok(())
    }
}
