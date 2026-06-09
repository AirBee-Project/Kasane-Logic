use crate::{Error, SpatialIdCollection, UnaryOperator};

use super::FillDefault;

/// fill 系演算子を「普通のメソッド」として呼び出すための拡張トレイト。
pub trait FillOps: SpatialIdCollection {
    /// 値を持つ領域を包む最小範囲（F/X/Y の3次元AABB）の隙間へ既定値 `default` を割り当てる。
    ///
    /// もともと値があったセルはその値を保持し（既定値で上書きしない）、AABB の外側へは何も
    /// 書き込まない。結果は AABB が隙間なく埋まり、既存セルは元の値・隙間は既定値となる。
    /// 空集合に適用すると AABB が無いため空のまま返る。
    ///
    /// 結果は断片化（隣接同値セルが分かれる）したまま返す。
    ///
    /// # 動作例
    ///
    /// X 方向に離れた2セルの間（AABB の隙間）が既定値で埋まり、両端は元の値を保つ:
    /// ```
    /// use kasane_logic::{FillOps, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 0, 2, 0).unwrap(), 30);
    ///
    /// let filled = a.fill_default(7).unwrap();
    /// let at = |x| filled.get(&SingleId::new(20, 0, x, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(10), Some(7), Some(30)));
    /// ```
    fn fill_default(&self, default: Self::Value) -> Result<Self, Error> {
        FillDefault::execution::<Self, Self>(self, default)
    }
}

impl<C> FillOps for C where C: SpatialIdCollection {}
