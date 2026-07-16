use crate::{BinaryOperator, Error, SpatialIdCollection, UnaryOperator};
use alloc::boxed::Box;

/// 式全体を表現する型
pub enum Query<S: SpatialIdCollection, U: UnaryOperator, B: BinaryOperator> {
    /// 演算の起点となるデータ
    Source(S),
    /// 単項演算
    Unary(U, Box<Query<S, U, B>>),
    // 二項演算
    Binary(B, Box<Query<S, U, B>>, Box<Query<S, U, B>>),
}

impl<S: SpatialIdCollection, U: UnaryOperator, B: BinaryOperator> Query<S, U, B> {
    #[allow(dead_code)]
    fn from(collection: S) -> Self {
        collection.query()
    }
}

impl<S: SpatialIdCollection, U: UnaryOperator, B: BinaryOperator> Query<S, U, B>
where
    S::Value: 'static,
{
    /// 最適化して[Query]を実行
    pub fn run(self) -> Result<S, Error> {
        match self {
            Query::Source(collection) => Ok(collection),
            Query::Unary(op, input) => {
                let mut input = input.run()?;
                op.run(&mut input).unwrap();
                Ok(input)
            }
            Query::Binary(op, lhs, rhs) => {
                #[cfg(feature = "rayon")]
                let (lhs_res, rhs_res) = rayon::join(|| lhs.run(), || rhs.run());

                #[cfg(not(feature = "rayon"))]
                let (lhs_res, rhs_res) = (lhs.run(), rhs.run());

                let mut lhs_res = lhs_res?;
                let rhs_res = rhs_res?;
                op.run(&mut lhs_res, &rhs_res).unwrap();
                Ok(lhs_res)
            }
        }
    }
}
