use core::ops::Add as StdAdd;

use crate::{BinaryOperator, Error, SpatialIdCollection};

use super::add::Add;

/// 算術演算を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// 集合演算（[`SetOps`](crate::SetOps)）がセルの帰属（構造）を扱うのに対し、こちらは重なった
/// セルの値を算術的に合成する。両コレクションは同じ値型を持つ必要がある。
pub trait ArithOps: SpatialIdCollection {
    /// 加算（A + B）。両方に値があるセルは値同士を足し合わせ、片側にしか値がないセルはその値を
    /// そのまま残す（欠落側を `0` とみなす和）。値型が [`core::ops::Add`] を実装する場合に使える。
    ///
    /// 結果は断片化（隣接同値セルが分かれる）したまま返す。
    ///
    /// # 動作例
    ///
    /// ```
    /// use kasane_logic::{ArithOps, SingleId, SpatialIdTable};
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
    fn add(&self, other: &Self) -> Result<Self, Error>
    where
        Self::Value: StdAdd<Output = Self::Value>,
    {
        Add::execution::<Self, Self, Self>(self, other, ())
    }
}

impl<C> ArithOps for C where C: SpatialIdCollection {}
