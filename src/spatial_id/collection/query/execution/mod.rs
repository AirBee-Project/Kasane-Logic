use super::traits::{BinaryOperator, UnaryOperator};
use crate::spatial_id::collection::query::execution::group_commutative::types::CommutativityInfo;
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
}

/// [`Query`] の木構造を人間が読める形式で出力する。
///
/// # 出力例（最適化前）
/// ```text
/// Source
/// → extrude_f(z=25, f=[0, 5], Max)
/// → falloff_linear_x(z=25, r=3, Max)
/// → falloff_linear_y(z=25, r=3, Max)
/// → falloff_linear_f(z=25, r=2, Max)
/// → fill_empty
/// ```
///
/// # 出力例（[`Query::optimize`] 後）
/// ```text
/// Source
/// → extrude_f(z=25, f=[0, 5], Max)
/// → [Group]
///     falloff_linear_f(z=25, r=2, Max)
///     falloff_linear_x(z=25, r=3, Max)
///     falloff_linear_y(z=25, r=3, Max)
/// → fill_empty
/// ```
///
/// # 出力例（Binary あり）
/// ```text
/// merge(Max)
///   lhs:
///     Source
///     → shift_x(z=25, x=3)
///   rhs:
///     Source
///     → falloff_linear_f(z=25, r=2, Max)
/// ```
impl<S: SpatialIdCollection> core::fmt::Display for Query<S>
where
    S::Value: 'static,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt_query(self, f, "")
    }
}

/// [`Query`] を再帰的に整形して `fmt` へ書き出す。
///
/// `indent` は現在の深さに対するインデント文字列（Binary の入れ子に使用）。
fn fmt_query<S: SpatialIdCollection>(
    query: &Query<S>,
    f: &mut core::fmt::Formatter<'_>,
    indent: &str,
) -> core::fmt::Result
where
    S::Value: 'static,
{
    match query {
        Query::Source(_) => write!(f, "{indent}Source"),

        Query::Unary(ops, input) => {
            fmt_query(input, f, indent)?;
            for op in ops {
                write!(f, "\n{indent}→ ")?;
                op.fmt_op(f)?;
            }
            Ok(())
        }

        Query::CommutativeGroup(_, ops, input) => {
            fmt_query(input, f, indent)?;
            write!(f, "\n{indent}→ [Group]")?;
            for op in ops {
                write!(f, "\n{indent}    ")?;
                op.fmt_op(f)?;
            }
            Ok(())
        }

        Query::Binary(op, lhs, rhs) => {
            // 二項演算子名を先頭に
            op.fmt_op(f)?;
            // lhs
            write!(f, "\n{indent}  lhs:\n")?;
            let lhs_indent = alloc::format!("{indent}    ");
            fmt_query(lhs, f, &lhs_indent)?;
            // rhs
            write!(f, "\n{indent}  rhs:\n")?;
            let rhs_indent = alloc::format!("{indent}    ");
            fmt_query(rhs, f, &rhs_indent)?;
            Ok(())
        }

        Query::Error(e) => write!(f, "{indent}Error({e:?})"),
    }
}
