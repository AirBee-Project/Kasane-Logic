use crate::spatial_id::collection::query::Query;
use crate::spatial_id::collection::query::cancellation::CancellationToken;
use crate::spatial_id::collection::query::query_iter::QueryIter;
use crate::spatial_id::collection::query::send_sync::{MaybeSendSync, SafeValue};
use crate::{Error, RangeId, SpatialId};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

/// [`Query::run`] に評価する範囲として渡せるもの。
///
/// `..` で全体、空間 ID 1つでその範囲、空間 ID の並び（配列・[`Vec`]・スライス）でそれらの和を表す。
pub trait IntoTargets {
    fn into_targets(self) -> Vec<RangeId>;
}

impl IntoTargets for core::ops::RangeFull {
    fn into_targets(self) -> Vec<RangeId> {
        vec![RangeId::WHOLE]
    }
}

impl<T: SpatialId> IntoTargets for T {
    fn into_targets(self) -> Vec<RangeId> {
        vec![self.into()]
    }
}

impl<T: SpatialId, const N: usize> IntoTargets for [T; N] {
    fn into_targets(self) -> Vec<RangeId> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: SpatialId> IntoTargets for Vec<T> {
    fn into_targets(self) -> Vec<RangeId> {
        self.into_iter().map(Into::into).collect()
    }
}

impl<T: SpatialId> IntoTargets for &[T] {
    fn into_targets(self) -> Vec<RangeId> {
        self.iter().cloned().map(Into::into).collect()
    }
}

/// クエリのデータを提供するTrait。[`SpatialIdTable`](crate::SpatialIdTable) やリモートのデータベースなどに実装する。
///
/// # 実装の仕方
///
/// [`read_flex_ids`](Self::read_flex_ids) は [`QueryIter`] の制約を満たす列を返す。
///
/// - FlexId の Z-order 列で読み出せるなら、[`QueryIter::from_ordered`] で包んで逐次に流す。
///   [`FlexId::zorder_key`](crate::FlexId::zorder_key) の昇順で保存したデータベースを、その順に一度だけ走査し、
///   `targets` のどれかと [`intersects_range`](crate::FlexId::intersects_range) する要素を返す場合がこれにあたる。
///   `targets` ごとに分けて読むと、複数の `targets` と重なる要素が重複し、順序も崩れうるので、
///   全ての `targets` をまとめて一度に読む。
pub trait Source: MaybeSendSync {
    type Value: SafeValue;

    /// `targets` のどれかと重なる要素を、それぞれ1回ずつ、[`QueryIter`] の制約を満たす列で返す。
    ///
    /// `targets` は互いに重なっていてもよく、順序も問わない。
    /// 返す要素は `targets` の外にはみ出していてもよい。
    fn read_flex_ids<'a>(
        &'a self,
        targets: &[RangeId],
        token: &CancellationToken,
    ) -> Result<QueryIter<'a, Self::Value>, Error>;

    fn query(self) -> Query<Self::Value>
    where
        Self: Sized + 'static,
    {
        Query::Source(Box::new(self))
    }
}

/// 二項演算子の定義。
pub trait BinaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    /// [`QueryIter`] の制約を満たす `lhs` と `rhs` を受け取り、二項演算した結果を同じ制約を満たす列で返す。
    fn run<'a>(
        &'a self,
        lhs: QueryIter<'a, V>,
        rhs: QueryIter<'a, V>,
    ) -> Result<QueryIter<'a, V>, Error>;
}

/// 単項演算子の定義。
pub trait UnaryOperator<V: SafeValue + 'static>: MaybeSendSync {
    /// パラメーターの事前検証
    fn validate(&self) -> Result<(), Error>;

    /// [`QueryIter`] の制約を満たす `input` を受け取り、単項演算した結果を同じ制約を満たす列で返す。
    fn run<'a>(&'a self, input: QueryIter<'a, V>) -> Result<QueryIter<'a, V>, Error>;

    /// 与えられた出力領域を計算するために必要な入力領域を `out` へ追加する。
    fn reverse_target(&self, output_target: RangeId, out: &mut Vec<RangeId>);
}
