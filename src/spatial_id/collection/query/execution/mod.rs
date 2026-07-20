use super::traits::{BinaryOperator, UnaryOperator};
use crate::{Error, SpatialIdCollection};
use alloc::boxed::Box;

/// 式全体を表現する型
pub enum Query<S: SpatialIdCollection> {
    /// 演算の起点となるデータ
    Source(S),
    /// 単項演算
    Unary(Box<dyn UnaryOperator<S::Working>>, Box<Query<S>>),
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
    /// 最適化して[Query]を実行する。
    ///
    /// 連鎖の**入口で 1 回** [`try_into_working`](SpatialIdCollection::try_into_working) し、全演算子を
    /// 作業表現 `S::Working` 上で回し、**出口で 1 回**
    /// [`try_from_working`](SpatialIdCollection::try_from_working) する。演算子ごとの再構築・（Table の）
    /// rank 再 intern を撤廃する。
    pub fn run(self) -> Result<S, Error> {
        S::try_from_working(self.run_core()?)
    }

    /// 作業表現を返す内部実行。連鎖の中間表現は `S::Working` のまま保たれる。
    fn run_core(self) -> Result<S::Working, Error> {
        match self {
            Query::Source(collection) => collection.try_into_working(),
            Query::Unary(op, input) => {
                let mut core = input.run_core()?;
                op.run(&mut core)?;
                Ok(core)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run_core(), || rhs.run_core());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run_core(), rhs.run_core());

                let mut lhs_res = lhs_res?;
                let rhs_res = rhs_res?;
                op.run(&mut lhs_res, &rhs_res)?;
                Ok(lhs_res)
            }
            Query::Error(e) => Err(e),
        }
    }
}
