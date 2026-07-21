use super::traits::{BinaryOperator, UnaryOperator, WorkingTree};
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
use crate::spatial_id::collection::query::lazy::LazyView;
use crate::{Error, SpatialIdCollection};
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

pub mod group_commutative;
pub mod sort_commutative;
pub mod validate;

#[cfg(test)]
mod test;

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
                    // 並び替えは AST 最適化 [`sort_commutative_ops`](Self::sort_commutative_ops)
                    // で事前に済んでいるので、ここでは与えられた順に実行するだけ。
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

    /// AST最適化を適用し、**実行しない**。
    ///
    /// ① [`group_commutative_ops`](Self::group_commutative_ops) で可換な区間を検知して
    /// `CommutativeGroup` にまとめ、② [`sort_commutative_ops`](Self::sort_commutative_ops) で
    /// 各グループ内の演算子を拡大率が小さい順へ並び替える。いずれも純粋な静的 AST 変換で、
    /// 最適化後の `Query` を `Display` で出力すればオプティマイザの挙動を実行なしで確認できる。
    /// 実行まで行う場合は [`run`](Self::run) を使う。
    ///
    /// ```text
    /// // 最適化前
    /// println!("{query}");
    /// // 最適化後（可換グループの検知・並び替えの結果を確認）
    /// println!("{}", query.optimize());
    /// ```
    pub fn optimize(self) -> Self
    where
        S::Working: 'static,
    {
        self.group_commutative_ops().sort_commutative_ops()
    }

    /// AST最適化（可換な区間の検知・グループ化・並び替え）を適用してから実行する。
    ///
    /// 実行前に [`validate`](Self::validate) でパラメータ・遅延エラーを検証するため、
    /// 最適化や実データ変換より先に構築時の問題を検出できる。
    /// 最適化のみ行いたい場合は [`optimize`](Self::optimize)、
    /// 最適化なしで実行したい場合は [`raw_run`](Self::raw_run) を使う。
    pub fn run(self) -> Result<S, Error>
    where
        S::Working: 'static,
    {
        self.validate()?;
        self.optimize().raw_run()
    }

    /// `bounds` で要求される最終結果を得るために必要な入力を `Query` ツリーの下位へと逆算していく。
    pub(crate) fn inverse_bounds(&self, bounds: crate::RangeId) -> Vec<crate::RangeId> {
        match self {
            Query::Source(_) => alloc::vec![bounds],
            Query::Unary(ops, input) => {
                let mut req = alloc::vec![bounds];
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        next.extend(op.inverse_bounds(r));
                    }
                    req = next;
                }
                let mut res = Vec::new();
                for r in req {
                    res.extend(input.inverse_bounds(r));
                }
                res
            }
            Query::CommutativeGroup(_, ops, input) => {
                let mut req = alloc::vec![bounds];
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        next.extend(op.inverse_bounds(r));
                    }
                    req = next;
                }
                let mut res = Vec::new();
                for r in req {
                    res.extend(input.inverse_bounds(r));
                }
                res
            }
            Query::Binary(op, lhs, rhs) => {
                let (inter_l, inter_r) = op.inverse_bounds(bounds);
                let mut req_l = Vec::new();
                for r in inter_l {
                    req_l.extend(lhs.inverse_bounds(r));
                }
                let mut req_r = Vec::new();
                for r in inter_r {
                    req_r.extend(rhs.inverse_bounds(r));
                }
                req_l.extend(req_r);
                req_l
            }
            Query::Error(_) => alloc::vec![bounds],
        }
    }
}

pub(crate) fn intersects_flex_range(flex: &crate::FlexId, range: &crate::RangeId) -> bool {
    fn intersect_axis(f_z: u8, f_i: i64, r_z: u8, r_min: i64, r_max: i64) -> bool {
        let (deep_z, deep_min, deep_max, shallow_z, shallow_min, shallow_max) = if f_z > r_z {
            (f_z, f_i, f_i, r_z, r_min, r_max)
        } else {
            (r_z, r_min, r_max, f_z, f_i, f_i)
        };
        let shift = deep_z - shallow_z;
        let deep_shallow_min = deep_min >> shift;
        let deep_shallow_max = deep_max >> shift;
        !(deep_shallow_max < shallow_min || deep_shallow_min > shallow_max)
    }

    intersect_axis(flex.f_zoomlevel(), flex.f_index() as i64, range.z(), range.f()[0] as i64, range.f()[1] as i64)
        && intersect_axis(flex.x_zoomlevel(), flex.x_index() as i64, range.z(), range.x()[0] as i64, range.x()[1] as i64)
        && intersect_axis(flex.y_zoomlevel(), flex.y_index() as i64, range.z(), range.y()[0] as i64, range.y()[1] as i64)
}

impl<S: SpatialIdCollection> Query<S> {
    pub(crate) fn run_on_subset(&self, bounds: &[crate::RangeId]) -> Result<S::Working, Error>
    where
        S::Working: 'static,
    {
        match self {
            Query::Source(s) => {
                let mut subset = Vec::new();
                for (id, val) in s.iter() {
                    if bounds.iter().any(|b| intersects_flex_range(&id, b)) {
                        subset.push((id, val.clone()));
                    }
                }
                Ok(S::Working::from_flexids(subset))
            }
            Query::Unary(ops, input) => {
                let mut req = bounds.to_vec();
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        next.extend(op.inverse_bounds(r));
                    }
                    req = next;
                }
                let mut working = input.run_on_subset(&req)?;
                for op in ops {
                    op.run(&mut working)?;
                }
                Ok(working)
            }
            Query::CommutativeGroup(_, ops, input) => {
                let mut req = bounds.to_vec();
                for op in ops.iter().rev() {
                    let mut next = Vec::new();
                    for r in req {
                        next.extend(op.inverse_bounds(r));
                    }
                    req = next;
                }
                let mut working = input.run_on_subset(&req)?;
                for op in ops {
                    op.run(&mut working)?;
                }
                Ok(working)
            }
            Query::Binary(op, lhs, rhs) => {
                let mut lhs_bounds = Vec::new();
                let mut rhs_bounds = Vec::new();
                for b in bounds {
                    let (l, r) = op.inverse_bounds(b.clone());
                    lhs_bounds.extend(l);
                    rhs_bounds.extend(r);
                }
                let mut lhs_working = lhs.run_on_subset(&lhs_bounds)?;
                let rhs_working = rhs.run_on_subset(&rhs_bounds)?;
                op.run(&mut lhs_working, &rhs_working)?;
                Ok(lhs_working)
            }
            Query::Error(e) => Err(e.clone()),
        }
    }

    /// クエリを評価せずに遅延ビュー（Lazy View）を作成する。
    pub fn lazy(&self) -> LazyView<'_, S> {
        LazyView { query: self }
    }
}
