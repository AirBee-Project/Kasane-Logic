#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use core::marker::PhantomData;

use crate::{BinaryOperator, Error};

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
