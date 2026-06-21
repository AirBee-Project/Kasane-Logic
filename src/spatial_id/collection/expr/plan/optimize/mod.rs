use alloc::boxed::Box;

use crate::SpatialIdCollection;

use super::Plan;

mod rules;

/// 書き換えの結果。ノードの所有権を返しつつ「変化したか」を伝える。
///
/// ルールは失敗時にもノードを返し戻す必要があるため、`Option` ではなくこの型を使う。
pub(crate) enum Rewrite<T> {
    /// ルールが適用され、別のノードへ書き換わった。
    Changed(T),
    /// ルールは適用されず、ノードはそのまま。
    Unchanged(T),
}

impl<T> Rewrite<T> {
    fn into_inner(self) -> T {
        match self {
            Rewrite::Changed(t) | Rewrite::Unchanged(t) => t,
        }
    }

    fn is_changed(&self) -> bool {
        matches!(self, Rewrite::Changed(_))
    }
}

/// 1 つのノードを局所的に書き換えるルール。子は既に最適化済みである前提で呼ばれる。
type Rule<C> = fn(Plan<C>) -> Rewrite<Plan<C>>;

impl<C: SpatialIdCollection> Plan<C>
where
    C::Value: 'static,
{
    /// 最適化を行う。
    ///
    /// 「木の巡回（ボトムアップ）」と「各ノードの書き換えルール」を分離している。
    pub fn optimize(self) -> Self {
        let rules: &[Rule<C>] = &[
            rules::drop_identity_unary,
            rules::fuse_adjacent_unary,
            rules::canonicalize_commutative,
        ];
        self.rewrite(rules)
    }

    /// 子を先に最適化してから、このノードへ安定するまでルールを適用する。
    fn rewrite(self, rules: &[Rule<C>]) -> Self {
        let node = match self {
            Plan::Source(collection) => Plan::Source(collection),
            Plan::Unary(op, input) => Plan::Unary(op, Box::new(input.rewrite(rules))),
            Plan::Binary(op, lhs, rhs) => Plan::Binary(
                op,
                Box::new(lhs.rewrite(rules)),
                Box::new(rhs.rewrite(rules)),
            ),
        };
        Self::apply_until_stable(node, rules)
    }

    /// このノードへ全ルールを順に当て、どれも変化させなくなるまで繰り返す。
    fn apply_until_stable(mut node: Self, rules: &[Rule<C>]) -> Self {
        loop {
            let mut changed = false;
            for rule in rules {
                let result = rule(node);
                changed |= result.is_changed();
                node = result.into_inner();
            }
            if !changed {
                break;
            }
        }
        node
    }

    /// プランに含まれるノード数。可換演算の正規化（重い部分木を左へ寄せる）に使う。
    fn node_count(&self) -> usize {
        match self {
            Plan::Source(_) => 1,
            Plan::Unary(_, input) => 1 + input.node_count(),
            Plan::Binary(_, lhs, rhs) => 1 + lhs.node_count() + rhs.node_count(),
        }
    }
}
