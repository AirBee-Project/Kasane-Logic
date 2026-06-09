use core::marker::PhantomData;

use crate::{BinaryOperator, Error};

/// 異型 `A` × `B` → `C` の総合口。4つの集合演算はこの特殊形にあたる。
///
/// クロージャ `f: (Option<&A>, Option<&B>) -> Option<C>` が4状態のうち `both_some`/`a_only`/
/// `b_only` の3つ（`(None, None)` は呼ばれない）を一手に引き受ける。`None` を返したセルは
/// 結果から除外される。任意関数のため可換性は保証できず、常に非可換とみなす。
///
/// クロージャ型 `F` と結果型 `C` は trait のパラメータにも self 型にも現れないため、
/// マーカーに `PhantomData` として持たせて型推論を成立させる（値としては生成しない）。
pub struct Combine<F, C>(PhantomData<(F, C)>);

impl<A, B, C, F> BinaryOperator<A, B> for Combine<F, C>
where
    A: Ord + PartialEq + Clone,
    B: Ord + PartialEq + Clone,
    C: Ord + PartialEq + Clone,
    F: Fn(Option<&A>, Option<&B>) -> Option<C>,
{
    type CustomParameter = F;
    type ResultValue = C;

    fn both_some(a: &A, b: &B, f: &Self::CustomParameter) -> Result<Option<C>, Error> {
        Ok(f(Some(a), Some(b)))
    }

    fn a_only(a: &A, f: &Self::CustomParameter) -> Result<Option<C>, Error> {
        Ok(f(Some(a), None))
    }

    fn b_only(b: &B, f: &Self::CustomParameter) -> Result<Option<C>, Error> {
        Ok(f(None, Some(b)))
    }

    fn is_commutative(_f: &Self::CustomParameter) -> bool {
        false
    }
}
