use core::marker::PhantomData;

use crate::spatial_id::collection::query::Query;
use crate::{BinaryOperator, CellValue, ConflictPolicy, Error, SpatialIdCollection};

/// 和集合（A ∪ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合は[ConflictPolicy]に従って合成する。
/// - Aのみの場合はAが残る。
/// - Bのみの場合はBが残る。
///
/// # 性質
/// - 可換性：[ConflictPolicy::Min]か[ConflictPolicy::Max]の場合に可換
pub struct Union;

impl<V: CellValue> BinaryOperator<V, V> for Union {
    type CustomParameter = ConflictPolicy<V>;
    type ResultValue = V;

    fn both_some(a: &V, b: &V, policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(policy.resolve(Some(a.clone()), b.clone())))
    }

    fn a_only(a: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }

    fn is_commutative(policy: &Self::CustomParameter) -> bool {
        matches!(policy, ConflictPolicy::Min | ConflictPolicy::Max)
    }
}

/// 積集合（A ∩ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合は[ConflictPolicy]に従って合成する。
/// - どちらかが存在しない場合はNoneとなる。
///
/// # 性質
/// - 可換性：[ConflictPolicy::Min]か[ConflictPolicy::Max]の場合に可換
pub struct Intersection;

impl<V: CellValue> BinaryOperator<V, V> for Intersection {
    type CustomParameter = ConflictPolicy<V>;
    type ResultValue = V;

    fn both_some(a: &V, b: &V, policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(policy.resolve(Some(a.clone()), b.clone())))
    }

    fn a_only(_a: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn b_only(_b: &V, _policy: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn is_commutative(policy: &Self::CustomParameter) -> bool {
        matches!(policy, ConflictPolicy::Min | ConflictPolicy::Max)
    }
}

/// 差集合（A - B）を行う二項演算。
///
/// # 計算内容
/// - Bの値がない場所にAの値を残す。
///
/// # 性質
/// - 可換性：非可換
pub struct Difference;

impl<A: CellValue, B: CellValue> BinaryOperator<A, B> for Difference {
    type CustomParameter = ();
    type ResultValue = A;

    fn both_some(_a: &A, _b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn a_only(a: &A, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(_b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        false
    }
}

/// 対称差（A △ B）を行う二項演算。
///
/// # 計算内容
/// - AとBが両方存在する場合はNoneにする。
/// - Aのみの場合はAが残る。
/// - Bのみの場合はBが残る。
///
/// # 性質
/// - 可換性：可換
pub struct SymmetricDifference;

impl<V: CellValue> BinaryOperator<V, V> for SymmetricDifference {
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(_a: &V, _b: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn a_only(a: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _p: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        true
    }
}

/// マスク（AをBの存在範囲で切り取る）二項演算。
///
/// # 計算内容
/// - Bに値が存在する部分をNoneにしたAを返す。
///
/// # 性質
/// - 可換性：非可換
pub struct Mask;

impl<A: CellValue, B: CellValue> BinaryOperator<A, B> for Mask {
    type CustomParameter = ();
    type ResultValue = A;

    fn both_some(a: &A, _b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(Some(a.clone()))
    }

    fn a_only(_a: &A, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn b_only(_b: &B, _p: &Self::CustomParameter) -> Result<Option<A>, Error> {
        Ok(None)
    }

    fn is_commutative(_p: &Self::CustomParameter) -> bool {
        false
    }
}

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

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    pub fn union_with(
        self,
        other: impl Into<Query<C>>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Union(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn intersection_with(
        self,
        other: impl Into<Query<C>>,
        conflict: ConflictPolicy<C::Value>,
    ) -> Self {
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Intersection(conflict),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn difference(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Difference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn symmetric_difference(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::SymmetricDifference,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }

    pub fn mask(self, other: impl Into<Query<C>>) -> Self {
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Mask,
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::{ConflictPolicy, FlexId, SingleId, SpatialIdSet, SpatialIdTable};

    fn id(z: u8, f: i32, x: u32, y: u32) -> SingleId {
        SingleId::new(z, f, x, y).unwrap()
    }

    fn value_at(t: &SpatialIdTable<u8>, z: u8, f: i32, x: u32, y: u32) -> Option<u8> {
        t.get(&id(z, f, x, y)).next().map(|(_, v)| *v)
    }

    fn present_b(t: &SpatialIdTable<bool>, z: u8, f: i32, x: u32, y: u32) -> bool {
        t.get(&id(z, f, x, y)).next().is_some()
    }

    // 共通の入力: a = {f0:1, f1:2}, b = {f1:9, f2:3}（f1 で重なる）。
    fn a_table() -> SpatialIdTable<u8> {
        let mut a = SpatialIdTable::new();
        a.insert(id(25, 0, 100, 100), 1);
        a.insert(id(25, 1, 100, 100), 2);
        a
    }

    fn b_table() -> SpatialIdTable<u8> {
        let mut b = SpatialIdTable::new();
        b.insert(id(25, 1, 100, 100), 9);
        b.insert(id(25, 2, 100, 100), 3);
        b
    }

    #[test]
    fn union_keeps_both_sides_and_resolves_overlap() {
        let u = a_table()
            .query()
            .union_with(b_table(), ConflictPolicy::Max)
            .run()
            .unwrap();

        assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1)); // a のみ
        assert_eq!(value_at(&u, 25, 1, 100, 100), Some(9)); // both → max(2,9)
        assert_eq!(value_at(&u, 25, 2, 100, 100), Some(3)); // b のみ
    }

    #[test]
    fn intersection_keeps_overlap_only() {
        let i = a_table()
            .query()
            .intersection_with(b_table(), ConflictPolicy::Min)
            .run()
            .unwrap();

        assert_eq!(value_at(&i, 25, 0, 100, 100), None);
        assert_eq!(value_at(&i, 25, 1, 100, 100), Some(2)); // both → min(2,9)
        assert_eq!(value_at(&i, 25, 2, 100, 100), None);
    }

    #[test]
    fn difference_removes_overlap() {
        let d = a_table().query().difference(b_table()).run().unwrap();

        assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1)); // a のみ → 残る
        assert_eq!(value_at(&d, 25, 1, 100, 100), None); // 重なり → 削る
        assert_eq!(value_at(&d, 25, 2, 100, 100), None); // b 側は無視
    }

    #[test]
    fn difference_accepts_table_as_mask() {
        let mut mask: SpatialIdTable<u8> = SpatialIdTable::new();
        mask.insert(id(25, 1, 100, 100), 0);

        let d = a_table().query().difference(mask).run().unwrap();
        assert_eq!(value_at(&d, 25, 0, 100, 100), Some(1));
        assert_eq!(value_at(&d, 25, 1, 100, 100), None);
    }

    #[test]
    fn symmetric_difference_keeps_exclusive_cells() {
        let x = a_table()
            .query()
            .symmetric_difference(b_table())
            .run()
            .unwrap();

        assert_eq!(value_at(&x, 25, 0, 100, 100), Some(1)); // a のみ
        assert_eq!(value_at(&x, 25, 1, 100, 100), None); // 重なり → 削る
        assert_eq!(value_at(&x, 25, 2, 100, 100), Some(3)); // b のみ
    }

    #[test]
    fn mask_keeps_left_value_on_overlap() {
        let mut region: SpatialIdTable<u8> = SpatialIdTable::new();
        region.insert(id(25, 1, 100, 100), 0);

        let m = a_table().query().mask(region).run().unwrap();
        assert_eq!(value_at(&m, 25, 0, 100, 100), None); // 範囲外 → 落ちる
        assert_eq!(value_at(&m, 25, 1, 100, 100), Some(2)); // a の値を保持
    }

    #[test]
    fn empty_identities() {
        let a = a_table();
        let empty: SpatialIdTable<u8> = SpatialIdTable::new();

        // A ∪ ∅ = A
        let u = a
            .clone()
            .query()
            .union_with(empty.clone(), ConflictPolicy::Max)
            .run()
            .unwrap();
        assert_eq!(value_at(&u, 25, 0, 100, 100), Some(1));
        assert_eq!(value_at(&u, 25, 1, 100, 100), Some(2));

        // A ∩ ∅ = ∅
        assert!(
            a.clone()
                .query()
                .intersection_with(empty.clone(), ConflictPolicy::Max)
                .run()
                .unwrap()
                .is_empty()
        );

        // A ∖ ∅ = A
        assert_eq!(
            value_at(
                &a.clone().query().difference(empty.clone()).run().unwrap(),
                25,
                0,
                100,
                100
            ),
            Some(1)
        );

        // A ∖ A = ∅
        assert!(
            a.clone()
                .query()
                .difference(a_table().clone())
                .run()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn difference_splits_coarse_cell() {
        // A は x が粗いセル（z24 index50 = z25 の x=100,101 を覆う）。B は内側の1セル(x=100)。
        // A ∖ B は残り（x=101）を細分して返す。
        let mut a: SpatialIdTable<bool> = SpatialIdTable::new();
        a.insert(FlexId::new(25, 0, 24, 50, 25, 100).unwrap(), true);
        let mut b: SpatialIdTable<bool> = SpatialIdTable::new();
        b.insert(id(25, 0, 100, 100), true);

        let d = a.clone().query().difference(b.clone()).run().unwrap();
        assert!(present_b(&d, 25, 0, 101, 100)); // 残り
        assert!(!present_b(&d, 25, 0, 100, 100)); // くり抜かれた
    }

    #[test]
    fn works_on_sets() {
        // Set 同士の集合演算（値なし）。
        let mut a = SpatialIdSet::new();
        a.insert(id(25, 0, 100, 100));
        a.insert(id(25, 1, 100, 100));
        let mut b = SpatialIdSet::new();
        b.insert(id(25, 1, 100, 100));

        let d = a.clone().query().difference(b.clone()).run().unwrap();
        assert!(d.get(&id(25, 0, 100, 100)).next().is_some());
        assert!(d.get(&id(25, 1, 100, 100)).next().is_none());
    }
}
