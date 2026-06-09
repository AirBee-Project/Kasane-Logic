use core::ops::{Add as StdAdd, Mul as StdMul, Sub as StdSub};

use crate::{BinaryOperator, Error, SpatialIdCollection};

use super::add::Add;
use super::mul::Mul;
use super::sub::Sub;

/// 加算を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 値型が [`core::ops::Add`] を実装するコレクションにのみ自動実装される。値を持たない
/// `Set`（`Value = ()`）のように `Add` を満たさない型には**実装されない**ため、型レベルで
/// 「加算できる集合」と「できない集合」を区別できる。
pub trait Addable: SpatialIdCollection
where
    Self::Value: StdAdd<Output = Self::Value>,
{
    /// 加算（A + B）。両方に値があるセルは値同士を足し合わせ、片側にしか値がないセルはその値を
    /// そのまま残す（欠落側を `0` とみなす和）。
    ///
    /// 結果は断片化（隣接同値セルが分かれる）したまま返す。
    ///
    /// # 動作例
    ///
    /// ```
    /// use kasane_logic::{Addable, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut b: SpatialIdTable<i32> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 5);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), 3);
    ///
    /// let s = a.add(&b).unwrap();
    /// let at = |f| s.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(10), Some(25), Some(3)));
    /// ```
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

/// 減算を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 値型が [`core::ops::Sub`] を実装するコレクションにのみ自動実装される。値を持たない
/// `Set`（`Value = ()`）には**実装されない**。
pub trait Subtractable: SpatialIdCollection
where
    Self::Value: StdSub<Output = Self::Value>,
{
    /// 減算（A - B）。両方に値があるセルは差 `a - b`、`a` だけのセルは `a` をそのまま残し、
    /// `b` だけのセルは結果に出さない（A の定義域内に結果をとどめる）。
    ///
    /// 結果は断片化（隣接同値セルが分かれる）したまま返す。
    ///
    /// # 動作例
    ///
    /// ```
    /// use kasane_logic::{Subtractable, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut b: SpatialIdTable<i32> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 5);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), 3);
    ///
    /// let d = a.sub(&b).unwrap();
    /// let at = |f| d.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(10), Some(15), None));
    /// ```
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

/// 乗算を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 値型が [`core::ops::Mul`] を実装するコレクションにのみ自動実装される。値を持たない
/// `Set`（`Value = ()`）には**実装されない**。
pub trait Multipliable: SpatialIdCollection
where
    Self::Value: StdMul<Output = Self::Value>,
{
    /// 乗算（A × B）。両方に値があるセルだけ積 `a * b` を残し、片側にしか値がないセルは結果に
    /// 出さない（欠落側を `0` とみなす積）。
    ///
    /// 結果は断片化（隣接同値セルが分かれる）したまま返す。
    ///
    /// # 動作例
    ///
    /// ```
    /// use kasane_logic::{Multipliable, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut b: SpatialIdTable<i32> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 5);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), 3);
    ///
    /// let m = a.mul(&b).unwrap();
    /// let at = |f| m.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (None, Some(100), None));
    /// ```
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
