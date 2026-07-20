use super::Query;
use crate::{Error, SpatialIdCollection};

impl<S: SpatialIdCollection> Query<S>
where
    S::Value: 'static,
{
    /// クエリ実行前に、AST内のすべての遅延エラー（`Query::Error`）および全演算子のパラメータ事前検証を行う。
    ///
    /// ASTノードを再帰的に巡回し、構築時エラー (`Query::Error`) や各演算子の `validate()` に
    /// 違反している場合、データ変換や重い演算処理を行う前にエラーを返します。
    pub fn validate(&self) -> Result<(), Error>
    where
        S::Working: 'static,
    {
        match self {
            Query::Source(_) => Ok(()),
            Query::Unary(ops, input) => {
                input.validate()?;
                for op in ops {
                    op.validate()?;
                }
                Ok(())
            }
            Query::CommutativeGroup(_, ops, input) => {
                input.validate()?;
                for op in ops {
                    op.validate()?;
                }
                Ok(())
            }
            Query::Binary(op, lhs, rhs) => {
                lhs.validate()?;
                rhs.validate()?;
                op.validate()
            }
            Query::Error(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod test;
