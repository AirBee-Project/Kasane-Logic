use super::Query;
use crate::Error;
use crate::spatial_id::collection::query::traits::WorkingTree;

impl<W: WorkingTree + 'static> Query<W>
where
    W::Value: 'static,
{
    /// クエリ実行前に、AST内のすべての遅延エラー（`Query::Error`）および全演算子のパラメータ事前検証を行う。   
    pub fn validate(&self) -> Result<(), Error> {
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
