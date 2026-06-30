use super::Query;
use crate::SpatialIdCollection;
mod rules;

impl<C: SpatialIdCollection> Query<C>
where
    C::Value: 'static,
{
    /// 最適化を行う。
    pub fn optimize(self) -> Self {
        // 最適化のルール一覧
        // プリミティブなルールほど最初のほうに置く

        #[allow(clippy::type_complexity)]
        let rules: &[fn(Query<C>) -> Query<C>] = &[rules::drop_identity_unary];
        let mut reuslt = self;

        for rule in rules {
            reuslt = rule(reuslt)
        }

        reuslt
    }

    /// プランに含まれるノード数を返す
    pub fn node_count(&self) -> usize {
        match self {
            Query::Source(_) => 1,
            Query::Unary(_, input) => 1 + input.node_count(),
            Query::Binary(_, lhs, rhs) => 1 + lhs.node_count() + rhs.node_count(),
        }
    }
}
