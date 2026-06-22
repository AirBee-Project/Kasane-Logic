use super::Plan;
use crate::SpatialIdCollection;
mod rules;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    /// 最適化を行う。
    pub fn optimize(self) -> Self {
        // 最適化のルール一覧
        // プリミティブなルールほど最初のほうに置く

        #[allow(clippy::type_complexity)]
        let rules: &[fn(Plan<C>) -> Plan<C>] = &[rules::drop_identity_unary];
        let mut reuslt = self;

        for rule in rules {
            reuslt = rule(reuslt)
        }

        reuslt
    }

    /// プランに含まれるノード数を返す
    pub fn node_count(&self) -> usize {
        match self {
            Plan::Source(_) => 1,
            Plan::Unary(_, input) => 1 + input.node_count(),
            Plan::Binary(_, lhs, rhs) => 1 + lhs.node_count() + rhs.node_count(),
        }
    }
}
