use crate::{SpatialIdCollection, spatial_id::collection::query::execution::Query};

impl<S: SpatialIdCollection> core::fmt::Display for Query<S>
where
    S::Value: 'static,
{
    /// [`Query`] の木構造を人間が読める形式で出力する。
    ///
    /// # 出力例（最適化前）
    /// ```text
    /// Source
    /// → extrude_f(z=25, f=[0, 5], Max)
    /// → falloff_linear_x(z=25, r=3, Max)
    /// → falloff_linear_y(z=25, r=3, Max)
    /// → falloff_linear_y(25, 1, Max)
    /// → extrude_f(25, 0, 0, Max)
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
