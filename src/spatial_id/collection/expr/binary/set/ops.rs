use crate::{BinaryOperator, ConflictPolicy, Error, SpatialIdCollection};

use super::combine::Combine;
use super::difference::Difference;
use super::intersection::Intersection;
use super::mask::Mask;
use super::symmetric_difference::SymmetricDifference;
use super::union::Union;

/// 集合演算を「普通のメソッド」として呼び出すための拡張トレイト。
///
/// セルがどちら側に属するか（構造）と、重なったときに値をどうするか（値合成）は直交する。
/// 値を持ち寄る [`union_with`](Self::union_with) / [`intersection_with`](Self::intersection_with)
/// は [`ConflictPolicy`] の指定を必須とし、値を混ぜない [`difference`](Self::difference) /
/// [`symmetric_difference`](Self::symmetric_difference) は方針を取らない。
///
/// 異なる値型のコレクションは次の2経路で扱う:
/// - [`difference`](Self::difference) / [`mask`](Self::mask) は相手を存在判定（presence）にのみ
///   使うため、相手の値型は問わない（`Set` や別型 `Table` をマスクにできる）。結果は自分の型を保つ。
/// - 真に両辺の値を別型のまま合成したいときは [`combine_with`](Self::combine_with) を使う。
///
/// 結果は断片化（隣接同値セルが分かれる）したまま返す。最小表現へまとめたい場合は別途
/// 正規化演算を適用する。各演算の具体例は個々のメソッドを参照。
pub trait SetOps: SpatialIdCollection {
    /// 和集合（A ∪ B）。両方に値があるセルは `policy` で畳み込む。
    ///
    /// # 動作例
    ///
    /// 重なるセルを大きい方で解決する和:
    /// ```
    /// use kasane_logic::{ConflictPolicy, SetOps, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<u8> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 2);
    /// let mut b: SpatialIdTable<u8> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 9);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), 3);
    ///
    /// let u = a.union_with(&b, ConflictPolicy::Max).unwrap();
    /// let at = |f| u.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(1), Some(9), Some(3)));
    /// ```
    fn union_with(&self, other: &Self, policy: ConflictPolicy<Self::Value>) -> Result<Self, Error> {
        Union::execution::<Self, Self, Self>(self, other, policy)
    }

    /// 積集合（A ∩ B）。重なるセルだけ残し、値は `policy` で畳み込む。
    ///
    /// # 動作例
    ///
    /// 重なるセルを小さい方で解決する積:
    /// ```
    /// use kasane_logic::{ConflictPolicy, SetOps, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<u8> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 2);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 5);
    /// let mut b: SpatialIdTable<u8> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 9);
    ///
    /// let i = a.intersection_with(&b, ConflictPolicy::Min).unwrap();
    /// let at = |f| i.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1)), (None, Some(5)));
    /// ```
    fn intersection_with(
        &self,
        other: &Self,
        policy: ConflictPolicy<Self::Value>,
    ) -> Result<Self, Error> {
        Intersection::execution::<Self, Self, Self>(self, other, policy)
    }

    /// 対称差（A △ B）。一方にしか値がないセルだけを残す（重なりは捨てる）。
    ///
    /// # 動作例
    ///
    /// 片側にしかないセルだけ残す:
    /// ```
    /// use kasane_logic::{SetOps, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<u8> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 1);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 2);
    /// let mut b: SpatialIdTable<u8> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), 9);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), 3);
    ///
    /// let x = a.symmetric_difference(&b).unwrap();
    /// let at = |f| x.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(1), None, Some(3)));
    /// ```
    fn symmetric_difference(&self, other: &Self) -> Result<Self, Error> {
        SymmetricDifference::execution::<Self, Self, Self>(self, other, ())
    }

    /// 差集合（A ∖ B）。`other` は存在判定（presence）にのみ使われるため任意の値型でよく、
    /// `Set` や別型 `Table` をマスクにできる。結果は自分（A）の値を保つ。
    ///
    /// # 動作例
    ///
    /// `Set` をマスクに使った差:
    /// ```
    /// use kasane_logic::{SetOps, SingleId, SpatialIdSet, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut mask = SpatialIdSet::new();
    /// mask.insert(SingleId::new(20, 1, 0, 0).unwrap());
    ///
    /// let d = a.difference(&mask).unwrap();
    /// let at = |f| d.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1)), (Some(10), None));
    /// ```
    fn difference<B>(&self, other: &B) -> Result<Self, Error>
    where
        B: SpatialIdCollection,
    {
        Difference::execution::<Self, B, Self>(self, other, ())
    }

    /// マスク（A を `other` の存在範囲で切り取る）。構造は積だが、重なるセルには常に A の値を残す。
    /// `other` は存在判定にのみ使われ任意の値型でよい。
    ///
    /// # 動作例
    ///
    /// `Set` の範囲で切り取り、A の値を保持:
    /// ```
    /// use kasane_logic::{SetOps, SingleId, SpatialIdSet, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut region = SpatialIdSet::new();
    /// region.insert(SingleId::new(20, 1, 0, 0).unwrap());
    ///
    /// let m = a.mask(&region).unwrap();
    /// let at = |f| m.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1)), (None, Some(20)));
    /// ```
    fn mask<B>(&self, other: &B) -> Result<Self, Error>
    where
        B: SpatialIdCollection,
    {
        Mask::execution::<Self, B, Self>(self, other, ())
    }

    /// 異型 A × B → C の総合合成。`f` が `both`/`a_only`/`b_only`（`(None, None)` 以外）を一手に
    /// 引き受け、`None` を返したセルは結果から除外される。4つの集合演算はこの特殊形にあたる。
    ///
    /// # パラメーター
    /// * `other` — 右辺コレクション（値型 `B` は自分と異なってよい）。
    /// * `f` — `(Option<&A>, Option<&B>) -> Option<C>`。出力型 `C` は結果コレクション `R` の値型。
    ///
    /// # 動作例
    ///
    /// `Table<i32>` と `Table<bool>` を `i32` へ合成する:
    /// ```
    /// use kasane_logic::{SetOps, SingleId, SpatialIdTable};
    /// let mut a: SpatialIdTable<i32> = SpatialIdTable::new();
    /// a.insert(SingleId::new(20, 0, 0, 0).unwrap(), 10);
    /// a.insert(SingleId::new(20, 1, 0, 0).unwrap(), 20);
    /// let mut b: SpatialIdTable<bool> = SpatialIdTable::new();
    /// b.insert(SingleId::new(20, 1, 0, 0).unwrap(), true);
    /// b.insert(SingleId::new(20, 2, 0, 0).unwrap(), true);
    ///
    /// let r: SpatialIdTable<i32> = a
    ///     .combine_with(&b, |av, bv| match (av, bv) {
    ///         (Some(a), Some(_)) => Some(a * 2), // 両方
    ///         (Some(a), None) => Some(*a),       // a のみ
    ///         (None, _) => None,                 // b のみ → 捨てる
    ///     })
    ///     .unwrap();
    /// let at = |f| r.get(&SingleId::new(20, f, 0, 0).unwrap()).next().map(|(_, v)| *v);
    /// assert_eq!((at(0), at(1), at(2)), (Some(10), Some(40), None));
    /// ```
    fn combine_with<B, R, F>(&self, other: &B, f: F) -> Result<R, Error>
    where
        B: SpatialIdCollection,
        R: SpatialIdCollection,
        F: Fn(Option<&Self::Value>, Option<&B::Value>) -> Option<R::Value>,
    {
        Combine::<F, R::Value>::execution::<Self, B, R>(self, other, f)
    }
}

impl<C> SetOps for C where C: SpatialIdCollection {}
