use core::ops::{Add as StdAdd, Mul as StdMul, Sub as StdSub};

use crate::{BinaryOperator, Error, SpatialIdCollection};

use super::add::Add;
use super::mul::Mul;
use super::sub::Sub;

pub trait Addable: SpatialIdCollection
where
    Self::Value: StdAdd<Output = Self::Value>,
{
    /// 加算(A+B)を行う二項演算。
    ///
    /// # 計算内容
    /// - 両方に値がある場合は値同士を足し合わせる。
    /// - 片方にしか値がない場合はそれを維持する。（Noneを0として解釈する）
    ///
    /// # 性質
    /// - 可換性：可換
    fn add(&self, other: &Self) -> Result<Self, Error> {
        Add::execution::<Self, Self, Self>(self, other, ())
    }
}

impl<C> Addable for C
where
    C: SpatialIdCollection,
    C::Value: StdAdd<Output = C::Value>,
{
}

pub trait Subtractable: SpatialIdCollection
where
    Self::Value: StdSub<Output = Self::Value>,
{
    /// 減算（A-B）を行う二項演算。
    ///
    /// # 計算内容
    /// - 両方に値がある場合はA-Bを行う。
    /// - Aにしか値がない場合は維持する。（BのNoneを0として解釈する）
    /// - Bにしか値がない場合はNoneを出力する。（Aが存在しないため計算不能）
    ///
    /// # 性質
    /// - 可換性：非可換
    fn sub(&self, other: &Self) -> Result<Self, Error> {
        Sub::execution::<Self, Self, Self>(self, other, ())
    }
}

impl<C> Subtractable for C
where
    C: SpatialIdCollection,
    C::Value: StdSub<Output = C::Value>,
{
}

pub trait Multipliable: SpatialIdCollection
where
    Self::Value: StdMul<Output = Self::Value>,
{
    /// 乗算(A×B)を行う二項演算。
    ///
    /// # 計算内容
    /// - 両方に値がある場合は値同士を掛け合わせる。
    /// - 片方にのみ値がある場合は0となる。(Noneを0として解釈する)
    ///
    /// # 性質
    /// - 可換性：可換
    fn mul(&self, other: &Self) -> Result<Self, Error> {
        Mul::execution::<Self, Self, Self>(self, other, ())
    }
}

impl<C> Multipliable for C
where
    C: SpatialIdCollection,
    C::Value: StdMul<Output = C::Value>,
{
}
