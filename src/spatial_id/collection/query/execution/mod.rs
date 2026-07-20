use super::traits::{BinaryOperator, UnaryOperator};
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::{Error, SpatialIdCollection};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub mod group_commutative;
pub mod validate;

/// 式全体を表現する型
pub enum Query<S: SpatialIdCollection> {
    /// 演算の起点となるデータ
    Source(S),
    /// 単項演算の直線区間（分岐の無い連続した単項演算子の列）。
    /// AST最適化（可換な演算の並び替え・同型演算子のmerge等）はこの `Vec` の中で完結する。
    Unary(Vec<Box<dyn UnaryOperator<S::Working>>>, Box<Query<S>>),
    /// 互いに可換な単項演算子のグループ
    CommutativeGroup(
        CommutativityInfo,
        Vec<Box<dyn UnaryOperator<S::Working>>>,
        Box<Query<S>>,
    ),
    // 二項演算
    Binary(
        Box<dyn BinaryOperator<S::Working>>,
        Box<Query<S>>,
        Box<Query<S>>,
    ),
    /// エラー状態を保持（AST構築時の遅延評価用）
    Error(Error),
}

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// `self` を単項演算子 `op` で包む。
    ///
    /// 直前のノードが `Unary` チェーンならその `Vec` へ追記し（実質的にチェーンを延長し）、
    /// そうでなければ `self` を新しい `Unary` ノードで包む。連続する単項演算子を1つの直線区間へ
    /// 自動的に集約するための構築ヘルパーで、各演算子のビルダーメソッド（`shift_x` 等）は
    /// `Query::Unary` を直接組み立てず、必ずこれを経由する（挙動を変えずに内部表現だけを配列化する
    /// ため）。
    ///
    /// `self` が `Query::Error` の場合の扱いは呼び出し側（各ビルダーメソッド）が事前に行う想定。
    pub(crate) fn wrap_unary<O>(self, op: O) -> Self
    where
        O: UnaryOperator<S::Working> + 'static,
    {
        match self {
            Query::Unary(mut ops, input) => {
                ops.push(Box::new(op));
                Query::Unary(ops, input)
            }
            other => Query::Unary(
                vec![Box::new(op) as Box<dyn UnaryOperator<S::Working>>],
                Box::new(other),
            ),
        }
    }

    /// なんの最適化もなく実行する
    pub fn raw_run(self) -> Result<S, Error>
    where
        S::Working: 'static,
    {
        fn run_internal<S: SpatialIdCollection>(query: Query<S>) -> Result<S::Working, Error>
        where
            S::Working: 'static,
        {
            match query {
                Query::Source(collection) => collection.try_into_working(),
                Query::Unary(ops, input) => {
                    let mut core = run_internal(*input)?;
                    for op in &ops {
                        op.run(&mut core)?;
                    }
                    Ok(core)
                }
                Query::CommutativeGroup(_, ops, input) => {
                    let mut core = run_internal(*input)?;
                    for op in &ops {
                        op.run(&mut core)?;
                    }
                    Ok(core)
                }
                Query::Binary(op, lhs, rhs) => {
                    #[cfg(feature = "rayon")]
                    let (lhs_res, rhs_res) =
                        rayon::join(|| run_internal(*lhs), || run_internal(*rhs));

                    #[cfg(not(feature = "rayon"))]
                    let (lhs_res, rhs_res) = (run_internal(*lhs), run_internal(*rhs));

                    let mut lhs_res = lhs_res?;
                    let rhs_res = rhs_res?;
                    op.run(&mut lhs_res, &rhs_res)?;
                    Ok(lhs_res)
                }
                Query::Error(e) => Err(e),
            }
        }
        S::try_from_working(run_internal(self)?)
    }
}
