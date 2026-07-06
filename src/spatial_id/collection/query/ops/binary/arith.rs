use core::ops::{Add as StdAdd, Mul as StdMul, Sub as StdSub};

use crate::spatial_id::collection::query::Query;
use crate::{BinaryOperator, CellValue, Error, SpatialIdCollection};

/// 加算(A+B)を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合は値同士を足し合わせる。
/// - 片方にしか値がない場合は存在した値を維持する。（Noneを0として解釈する）
///
/// # 性質
/// - 可換性：可換
pub struct Add;

impl<V> BinaryOperator<V, V> for Add
where
    V: CellValue + StdAdd<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() + b.clone()))
    }

    fn a_only(a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(b.clone()))
    }

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        true
    }
}

/// 減算（A-B）を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合はA-Bを行う。
/// - Aにしか値がない場合は維持する。（BのNoneを0として解釈する）
/// - Bにしか値がない場合はNoneを出力する。（Aが存在しないため計算不能）
///
/// # 性質
/// - 可換性：非可換
pub struct Sub;

impl<V> BinaryOperator<V, V> for Sub
where
    V: CellValue + StdSub<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() - b.clone()))
    }

    fn a_only(a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone()))
    }

    fn b_only(_b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        false
    }
}

/// 乗算(A×B)を行う二項演算。
///
/// # 計算内容
/// - 両方に値がある場合は値同士を掛け合わせる。
/// - 片方にのみ値がある場合は0となる。(Noneを0として解釈する)
///
/// # 性質
/// - 可換性：可換
pub struct Mul;

impl<V> BinaryOperator<V, V> for Mul
where
    V: CellValue + StdMul<Output = V>,
{
    type CustomParameter = ();
    type ResultValue = V;

    fn both_some(a: &V, b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(Some(a.clone() * b.clone()))
    }

    fn a_only(_a: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn b_only(_b: &V, _: &Self::CustomParameter) -> Result<Option<V>, Error> {
        Ok(None)
    }

    fn is_commutative(_: &Self::CustomParameter) -> bool {
        true
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdAdd<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::query::ops::binary::BinaryOpKernel::<Add, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdSub<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::query::ops::binary::BinaryOpKernel::<Sub, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static + StdMul<Output = C::Value>,
{
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: impl Into<Query<C>>) -> Self {
        let kernel = alloc::boxed::Box::new(
            crate::spatial_id::collection::query::ops::binary::BinaryOpKernel::<Mul, _> {
                param: (),
                _op: core::marker::PhantomData,
            },
        );
        Query::Binary(
            crate::spatial_id::collection::query::ops::binary::BinaryOp::Custom(kernel),
            alloc::boxed::Box::new(self),
            alloc::boxed::Box::new(other.into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::SpatialIdCollection;
    use crate::{SingleId, SpatialIdTable};

    fn id(z: u8, f: i32, x: u32, y: u32) -> SingleId {
        SingleId::new(z, f, x, y).unwrap()
    }

    fn value_at(t: &SpatialIdTable<i32>, z: u8, f: i32, x: u32, y: u32) -> Option<i32> {
        t.get(&id(z, f, x, y)).next().map(|(_, v)| *v)
    }

    // 共通の入力: a = {f0:10, f1:20}, b = {f1:5, f2:3}（f1 で重なる）。
    fn a_table() -> SpatialIdTable<i32> {
        let mut a = SpatialIdTable::new();
        a.insert(id(25, 0, 100, 100), 10);
        a.insert(id(25, 1, 100, 100), 20);
        a
    }

    fn b_table() -> SpatialIdTable<i32> {
        let mut b = SpatialIdTable::new();
        b.insert(id(25, 1, 100, 100), 5);
        b.insert(id(25, 2, 100, 100), 3);
        b
    }

    #[test]
    fn add_sums_overlap_and_keeps_each_side() {
        let s = a_table()
            .clone()
            .query()
            .add(b_table().clone())
            .run()
            .unwrap();

        assert_eq!(value_at(&s, 25, 0, 100, 100), Some(10)); // a のみ
        assert_eq!(value_at(&s, 25, 1, 100, 100), Some(25)); // both → 20 + 5
        assert_eq!(value_at(&s, 25, 2, 100, 100), Some(3)); // b のみ
    }

    #[test]
    fn add_is_commutative() {
        let ab = a_table()
            .clone()
            .query()
            .add(b_table().clone())
            .run()
            .unwrap();
        let ba = b_table()
            .clone()
            .query()
            .add(a_table().clone())
            .run()
            .unwrap();

        for f in 0..=2 {
            assert_eq!(
                value_at(&ab, 25, f, 100, 100),
                value_at(&ba, 25, f, 100, 100)
            );
        }
    }

    #[test]
    fn add_with_empty_is_identity() {
        let empty = SpatialIdTable::<i32>::new();
        let s = a_table().clone().query().add(empty.clone()).run().unwrap();

        assert_eq!(value_at(&s, 25, 0, 100, 100), Some(10));
        assert_eq!(value_at(&s, 25, 1, 100, 100), Some(20));
        assert_eq!(value_at(&s, 25, 2, 100, 100), None);
    }

    #[test]
    fn sub_keeps_a_domain_and_drops_b_only() {
        let d = a_table()
            .clone()
            .query()
            .sub(b_table().clone())
            .run()
            .unwrap();

        assert_eq!(value_at(&d, 25, 0, 100, 100), Some(10)); // a のみ → a
        assert_eq!(value_at(&d, 25, 1, 100, 100), Some(15)); // both → 20 - 5
        assert_eq!(value_at(&d, 25, 2, 100, 100), None); // b のみ → 捨てる
    }

    #[test]
    fn sub_self_is_zero_over_a_domain() {
        let d = a_table()
            .clone()
            .query()
            .sub(a_table().clone())
            .run()
            .unwrap();

        assert_eq!(value_at(&d, 25, 0, 100, 100), Some(0));
        assert_eq!(value_at(&d, 25, 1, 100, 100), Some(0));
    }

    #[test]
    fn mul_keeps_overlap_only() {
        let m = a_table()
            .clone()
            .query()
            .mul(b_table().clone())
            .run()
            .unwrap();

        assert_eq!(value_at(&m, 25, 0, 100, 100), None); // a のみ → 捨てる
        assert_eq!(value_at(&m, 25, 1, 100, 100), Some(100)); // both → 20 * 5
        assert_eq!(value_at(&m, 25, 2, 100, 100), None); // b のみ → 捨てる
    }

    #[test]
    fn mul_is_commutative() {
        let ab = a_table()
            .clone()
            .query()
            .mul(b_table().clone())
            .run()
            .unwrap();
        let ba = b_table()
            .clone()
            .query()
            .mul(a_table().clone())
            .run()
            .unwrap();

        for f in 0..=2 {
            assert_eq!(
                value_at(&ab, 25, f, 100, 100),
                value_at(&ba, 25, f, 100, 100)
            );
        }
    }

    #[test]
    fn mul_over_overlapping_ranges_at_mixed_zoom() {
        // 粗いセル（z24）に細かいセル（z25）が重なるケース。重なり部分のみ積が残る。
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(24, 0, 50, 50), 3); // z25 の f0,f1 を覆う粗いセル

        let mut b = SpatialIdTable::<i32>::new();
        b.insert(id(25, 0, 100, 100), 7); // a の被覆領域内の細かいセル

        let m = a.clone().query().mul(b.clone()).run().unwrap();

        // 重なるセルは 3 * 7、覆われていない残りは片側のみなので消える。
        assert_eq!(value_at(&m, 25, 0, 100, 100), Some(21));
        assert_eq!(value_at(&m, 25, 1, 100, 100), None);
    }

    #[test]
    fn add_over_overlapping_ranges_at_mixed_zoom() {
        // 粗いセル（z24）に細かいセル（z25）が重なるケース。重なり部分のみ加算される。
        let mut a = SpatialIdTable::<i32>::new();
        a.insert(id(24, 0, 50, 50), 100); // z25 の f0,f1 を覆う粗いセル

        let mut b = SpatialIdTable::<i32>::new();
        b.insert(id(25, 0, 100, 100), 1); // a の被覆領域内の細かいセル

        let s = a.clone().query().add(b.clone()).run().unwrap();

        // 重なるセルは 100 + 1、覆われていない残りは a の 100 のまま。
        assert_eq!(value_at(&s, 25, 0, 100, 100), Some(101));
        assert_eq!(value_at(&s, 25, 1, 100, 100), Some(100));
    }
}
