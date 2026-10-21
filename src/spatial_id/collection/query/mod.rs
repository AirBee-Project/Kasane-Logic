/// 値や演算子に課す `Send + Sync` の制約
pub(crate) mod send_sync;

/// クエリ実行を途中で打ち切るための協調的キャンセル
pub(crate) mod cancellation;

/// 演算子の種類
pub(crate) mod ops;

/// 演算子の間を流れる列
pub(crate) mod query_iter;

/// 演算子と入力源のTrait
pub(crate) mod traits;

/// 式全体を評価・実行するためのモジュール
pub(crate) mod execution;

/// 複数の値が同じ空間で衝突した際の解決ポリシー
pub mod merge_policy;

use crate::Error;
use crate::spatial_id::collection::query::send_sync::SafeValue;
use crate::spatial_id::collection::query::traits::{BinaryOperator, Source, UnaryOperator};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

/// Query全体を表現する型。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点
    Source(Box<dyn Source<Value = V>>),

    /// 連続した単項演算子
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),

    // 二項演算
    Binary(Box<dyn BinaryOperator<V>>, Box<Query<V>>, Box<Query<V>>),

    /// エラー状態を保持
    Error(Error),
}

// Queryの構築
impl<V: SafeValue + 'static> Query<V> {
    /// `self` を単項演算子で包む。
    pub(crate) fn wrap_unary<O>(self, op: O) -> Self
    where
        O: UnaryOperator<V> + 'static,
    {
        match self {
            Query::Unary(mut ops, input) => {
                ops.push(Box::new(op));
                Query::Unary(ops, input)
            }
            other => Query::Unary(
                vec![Box::new(op) as Box<dyn UnaryOperator<V>>],
                Box::new(other),
            ),
        }
    }
}

/// `Source` を実装する型を、二項演算子の引数などで直接 [`Query`] として渡せるようにする。
impl<V: SafeValue + 'static, S: Source<Value = V> + 'static> From<S> for Query<V> {
    fn from(source: S) -> Self {
        source.query()
    }
}
