use core::marker::PhantomData;

use crate::{BinaryOperator, CellValue, Error};

/// 異なる型を持つTableを合成するための二項演算。
///
/// # 計算内容
/// - 与えられた関数の通りにAとBを合成する。
///
/// # 性質
/// - 可換性：非可換
pub struct Combine<F, C>(PhantomData<(F, C)>);

impl<A, B, C, F> BinaryOperator<A, B> for Combine<F, C>
where
    A: CellValue,
    B: CellValue,
    C: CellValue,
    F: Fn(Option<&A>, Option<&B>) -> Option<C> + Sync,
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
