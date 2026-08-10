use super::traits::{BinaryOperator, UnaryOperator};
use crate::Error;
use crate::spatial_id::collection::flex_tree::core::SafeValue;
use crate::spatial_id::collection::query::execution::group_commutative::optimized_unary_order;
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::grid::try_run_grid;
use crate::spatial_id::collection::query::source::Source;
use crate::spatial_id::collection::query::working::WorkingTree;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub mod group_commutative;
pub mod sort_commutative;
pub mod validate;

#[cfg(test)]
mod test;

/// 式全体を表現する型。
///
/// 値型 `V` で直接パラメータ化されており、入力源の具象型には依存しない。
/// これにより、1つの AST 内でインメモリ入力源とディスク入力源を混在させられる
/// （例: [`Query::merge`] の左辺がディスク、右辺がインメモリ）。
pub enum Query<V: SafeValue + 'static> {
    /// 演算の起点となる入力源
    Source(Box<dyn Source<Value = V>>),
    /// 単項演算の直線区間（分岐の無い連続した単項演算子の列）。
    /// AST最適化（可換な演算の並び替え・同型演算子のmerge等）はこの `Vec` の中で完結する。
    Unary(Vec<Box<dyn UnaryOperator<V>>>, Box<Query<V>>),
    /// 互いに可換な単項演算子のグループ
    CommutativeGroup(
        CommutativityInfo,
        Vec<Box<dyn UnaryOperator<V>>>,
        Box<Query<V>>,
    ),
    // 二項演算
    Binary(Box<dyn BinaryOperator<V>>, Box<Query<V>>, Box<Query<V>>),
    /// エラー状態を保持（AST構築時の遅延評価用）
    Error(Error),
}

/// 単項演算の並びを作業木へ適用する。
///
/// [`grid_op`](UnaryOperator::grid_op) を持つ演算が連続している区間は、木を経由せず
/// 一様ズームの平坦表現（[`grid`](crate::spatial_id::collection::query::grid)）でまとめて
/// 処理する。中間木の構築が丸ごと消えるうえ、falloff は 2r+1 倍の中間データを作らずに
/// 出力を直接生成できる。平坦化の見積もりが予算を超えた場合は、その演算だけ従来どおり
/// 木の上で実行して次へ進む。
pub(crate) fn run_unary_chain<V: SafeValue + 'static>(
    mut ops: &[&dyn UnaryOperator<V>],
    mut working: WorkingTree<V>,
) -> Result<WorkingTree<V>, Error> {
    while let Some(head) = ops.first() {
        // グリッドで実行できる演算の最長区間を取る。
        let mut grid_len = 0;
        let mut max_z = None;
        for op in ops.iter() {
            if let Some(z) = op.grid_zoom() {
                grid_len += 1;
                max_z = Some(max_z.map_or(z, |m: crate::ZoomLevel| m.max(z)));
            } else {
                break;
            }
        }

        if grid_len > 0 {
            let grid_ops = &ops[..grid_len];
            if let Some(result) =
                try_run_grid(&working, grid_ops, max_z.unwrap(), grid_budget(&working))
            {
                working = result?;
                ops = &ops[grid_len..];
                continue;
            }
        }

        // グリッドに載らない（または載せる価値がない）ので木の上で実行する。
        head.run(&mut working)?;
        ops = &ops[1..];
    }
    Ok(working)
}

/// 平坦化を許す件数の上限。
///
/// 平坦化は葉をグリッドのズームまで細分するので、粗い葉が多いと件数が爆発する。
/// 一方の木経路は 1 葉あたり `2r+1` 個の中間 [`FlexId`](crate::FlexId) を作って挿入し
/// 直すので、数十倍までの膨張ならグリッドの方が確実に速い。葉数に比例した予算に
/// 小さな下駄を履かせ、それを超えるなら木経路へ譲る。
fn grid_budget<V: SafeValue>(working: &WorkingTree<V>) -> u64 {
    (working.count() as u64)
        .saturating_mul(64)
        .saturating_add(1 << 20)
}

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

    /// 検証も最適化もせず [`Query`] を実行し、作業木のまま返す。
    ///
    /// AST に書かれた順序をそのまま適用する（`raw_` はそれを表す）。最適化後の順序で
    /// 走らせたい場合は [`run_working_tree`](Self::run_working_tree) を使うこと。
    pub fn raw_run_working_tree(self) -> Result<WorkingTree<V>, Error> {
        fn run_internal<V: SafeValue + 'static>(query: Query<V>) -> Result<WorkingTree<V>, Error> {
            match query {
                Query::Source(source) => source.read_all(),
                Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
                    let order: Vec<&dyn UnaryOperator<V>> = ops.iter().map(|op| &**op).collect();
                    run_unary_chain(&order, run_internal(*input)?)
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
        run_internal(self)
    }

    /// AST最適化を適用する（実行は行わない）。
    pub fn optimize(self) -> Self {
        self.group_commutative_ops().sort_commutative_ops()
    }

    /// 検証・AST最適化を適用して実行し、作業木のまま返す。
    ///
    /// 具象コレクションで受け取りたい場合は [`run`](Self::run)（[`SpatialIdTable`]）や
    /// [`run_set`](Self::run_set)（[`SpatialIdSet`]）を使う。結果を走査するだけなら、
    /// 辞書への再 intern が無いぶんこちらのほうが速い。
    ///
    /// [`SpatialIdTable`]: crate::SpatialIdTable
    /// [`SpatialIdSet`]: crate::SpatialIdSet
    pub fn run_working_tree(self) -> Result<WorkingTree<V>, Error> {
        self.validate()?;
        self.optimize().raw_run_working_tree()
    }

    /// 出力領域 `bounds` を得るのに必要な入力領域を逆算しながら、その部分だけを評価する。
    ///
    /// AST 全体を評価してから絞るのではなく、各演算子の
    /// [`inverse_bounds`](UnaryOperator::inverse_bounds) で必要な入力領域を逆算し、
    /// [`Source::read_subset`] まで押し下げる。
    ///
    /// # [`run_working_tree`](Self::run_working_tree) との違い
    ///
    /// 評価範囲が違うだけで、**演算子の適用順は同じ**である。`&self` しか持てないので
    /// AST を組み替える [`optimize`](Self::optimize) は呼べないが、同じ
    /// `commutative_runs` を使って実行順だけを最適化後の順序へ揃えている。
    ///
    /// `&self` を取るので、領域を変えて何度でも呼べる（[`lazy_get`](Self::lazy_get) が
    /// この上に乗っている）。
    pub fn run_within(&self, bounds: Vec<crate::RangeId>) -> Result<WorkingTree<V>, Error> {
        self.validate()?;
        self.run_within_unchecked(bounds)
    }

    /// [`run_within`](Self::run_within) の本体（再帰部分）。
    fn run_within_unchecked(&self, bounds: Vec<crate::RangeId>) -> Result<WorkingTree<V>, Error> {
        match self {
            Query::Source(s) => s.read_subset(&bounds),
            Query::Unary(ops, input) | Query::CommutativeGroup(_, ops, input) => {
                // 逆算は AST に書かれた順（実行の逆順）で辿る。並べ替えは可換な区間の
                // 中でしか起きず、可換な演算子同士は必要入力領域も入れ替わらない。
                let mut req = bounds;
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        next.extend(op.inverse_bounds(r));
                    }
                    next.sort_unstable();
                    next.dedup();
                    req = next;
                }
                run_unary_chain(
                    &optimized_unary_order(ops),
                    input.run_within_unchecked(req)?,
                )
            }
            Query::Binary(op, lhs, rhs) => {
                let mut lhs_bounds = Vec::new();
                let mut rhs_bounds = Vec::new();
                for b in bounds {
                    let (l, r) = op.inverse_bounds(b.clone());
                    lhs_bounds.extend(l);
                    rhs_bounds.extend(r);
                }
                lhs_bounds.sort_unstable();
                lhs_bounds.dedup();
                rhs_bounds.sort_unstable();
                rhs_bounds.dedup();
                let mut lhs_working = lhs.run_within_unchecked(lhs_bounds)?;
                let rhs_working = rhs.run_within_unchecked(rhs_bounds)?;
                op.run(&mut lhs_working, &rhs_working)?;
                Ok(lhs_working)
            }
            Query::Error(e) => Err(e.clone()),
        }
    }

    /// 対象領域(`target`)と交差するSegmentだけを遅延評価して返す。
    pub fn lazy_get<T: crate::SpatialId>(
        &self,
        target: T,
    ) -> Result<impl Iterator<Item = (crate::FlexId, V)>, Error> {
        let working = self.run_within(vec![target.clone().into()])?;
        let target_range: crate::RangeId = target.into();

        Ok(working
            .into_iter()
            .filter(move |(id, _)| id.intersects_range(&target_range)))
    }

    /// 対象領域(`target`)のうち、データが存在しない空間を `default_value` で埋めてから値を返す。
    pub fn lazy_get_with_default<T: crate::SpatialId>(
        &self,
        target: T,
        default_value: V,
    ) -> Result<impl Iterator<Item = (crate::FlexId, V)>, Error> {
        let working = self.run_within(vec![target.clone().into()])?;
        let target_range: crate::RangeId = target.clone().into();

        let mut uncovered = crate::SpatialIdSet::new();
        uncovered.insert(target.into());

        let mut working_iter = working.into_iter();
        let mut default_iter = None;

        Ok(core::iter::from_fn(move || {
            if default_iter.is_none() {
                for (id, value) in working_iter.by_ref() {
                    if id.intersects_range(&target_range) {
                        uncovered.remove(&id);
                        return Some((id, value));
                    }
                }
                default_iter = Some(core::mem::take(&mut uncovered).into_iter());
            }

            default_iter
                .as_mut()?
                .next()
                .map(|(id, _)| (id, default_value.clone()))
        }))
    }
}
