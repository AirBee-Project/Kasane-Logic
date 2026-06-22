use alloc::vec::Vec;

use crate::{CellValue, Error, FlexId, SpatialIdCollection};

/// 空間IDコレクション同士から二項演算を行うTrait。
///
/// 2つのコレクションはいずれも「空間ID → 値」の部分関数であり、ある空間IDにおいて各々は
/// 値を持つ（`Some`）か持たない（`None`）。したがって同じ位置にある2つの空間IDの状態は
/// 次の4つに分かれる。
///
/// | `a` | `b` | 関数 |
/// |-----|-----|------|
/// | `Some` | `Some` | [`both_some`](Self::both_some) |
/// | `Some` | `None` | [`a_only`](Self::a_only) |
/// | `None` | `Some` | [`b_only`](Self::b_only) |
/// | `None` | `None` | そもそも演算を行わない |
pub trait BinaryOperator<A, B>
where
    A: CellValue,
    B: CellValue,
{
    #[cfg(feature = "rayon")]
    type CustomParameter: Sync;

    #[cfg(not(feature = "rayon"))]
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: CellValue;

    fn both_some(
        a: &A,
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    fn a_only(
        a: &A,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    fn b_only(
        b: &B,
        custom_parameter: &Self::CustomParameter,
    ) -> Result<Option<Self::ResultValue>, Error>;

    /// この演算が可換なのかを判定する。
    fn is_commutative(custom_parameter: &Self::CustomParameter) -> bool;

    fn execution<SA, SB, O>(
        a: &SA,
        b: &SB,
        custom_parameter: Self::CustomParameter,
    ) -> Result<O, Error>
    where
        SA: SpatialIdCollection<Value = A>,
        SB: SpatialIdCollection<Value = B>,
        O: SpatialIdCollection<Value = Self::ResultValue>,
    {
        // 参照で集める（値はクローンしない）。結果セルに必要なときだけ both_some 等で複製する。
        let a_cells: Vec<_> = a.scan_ref().collect();
        let b_cells: Vec<_> = b.scan_ref().collect();

        type MapResult<T> = Result<Vec<Vec<(crate::FlexId, T)>>, Error>;

        #[cfg(feature = "rayon")]
        let (new_from_a, new_from_b): (
            MapResult<Self::ResultValue>,
            MapResult<Self::ResultValue>,
        ) = {
            use rayon::prelude::*;
            rayon::join(
                || {
                    a_cells
                        .into_par_iter()
                        .map(|(a_id, a_value)| {
                            let mut local = Vec::new();
                            let mut covered = Vec::new();
                            for (overlap, b_value) in b.query_ref(&a_id) {
                                if let Some(value) =
                                    Self::both_some(a_value, b_value, &custom_parameter)?
                                {
                                    local.push((overlap.clone(), value));
                                }
                                covered.push(overlap);
                            }
                            for region in subtract_regions(a_id, &covered) {
                                if let Some(value) = Self::a_only(a_value, &custom_parameter)? {
                                    local.push((region, value));
                                }
                            }
                            Ok(local)
                        })
                        .collect()
                },
                || {
                    b_cells
                        .into_par_iter()
                        .map(|(b_id, b_value)| {
                            let mut local = Vec::new();
                            let covered: Vec<FlexId> =
                                a.query_ref(&b_id).map(|(id, _)| id).collect();
                            for region in subtract_regions(b_id, &covered) {
                                if let Some(value) = Self::b_only(b_value, &custom_parameter)? {
                                    local.push((region, value));
                                }
                            }
                            Ok(local)
                        })
                        .collect()
                },
            )
        };

        #[cfg(not(feature = "rayon"))]
        let (new_from_a, new_from_b): (
            MapResult<Self::ResultValue>,
            MapResult<Self::ResultValue>,
        ) = {
            let res_a: MapResult<Self::ResultValue> = a_cells
                .into_iter()
                .map(|(a_id, a_value)| {
                    let mut local = Vec::new();
                    let mut covered = Vec::new();
                    for (overlap, b_value) in b.query_ref(&a_id) {
                        if let Some(value) = Self::both_some(a_value, b_value, &custom_parameter)? {
                            local.push((overlap.clone(), value));
                        }
                        covered.push(overlap);
                    }
                    for region in subtract_regions(a_id, &covered) {
                        if let Some(value) = Self::a_only(a_value, &custom_parameter)? {
                            local.push((region, value));
                        }
                    }
                    Ok(local)
                })
                .collect();

            let res_b: MapResult<Self::ResultValue> = b_cells
                .into_iter()
                .map(|(b_id, b_value)| {
                    let mut local = Vec::new();
                    let covered: Vec<FlexId> = a.query_ref(&b_id).map(|(id, _)| id).collect();
                    for region in subtract_regions(b_id, &covered) {
                        if let Some(value) = Self::b_only(b_value, &custom_parameter)? {
                            local.push((region, value));
                        }
                    }
                    Ok(local)
                })
                .collect();

            (res_a, res_b)
        };

        // a 由来と b 由来の領域は構造上互いに素なので、衝突は起きない（Overwrite で十分）。
        let cells = new_from_a?
            .into_iter()
            .flatten()
            .chain(new_from_b?.into_iter().flatten());
        Ok(O::from_cells(cells, &ConflictPolicy::Overwrite))
    }
}

/// 同一の空間に複数の値が集まったときに、それらを1つの値にする方法。
pub enum ConflictPolicy<V> {
    /// 既存の値を保持し、後から来た候補を捨てる。
    KeepExisting,
    /// 後から来た候補で既存の値を上書きする。
    Overwrite,
    /// [`Ord`] 上で小さい方を採用する。
    Min,
    /// [`Ord`] 上で大きい方を採用する。
    Max,
    /// ユーザ定義の関数で合成する。引数は `(既存値, 新しい候補)` の順。
    Fold(fn(&V, &V) -> V),
}

impl<V: Ord> ConflictPolicy<V> {
    /// 既存値 `current`（無ければ `None`）に新しい候補 `incoming` を、方針に従って畳み込む。
    pub fn resolve(&self, current: Option<V>, incoming: V) -> V {
        let Some(current) = current else {
            return incoming;
        };

        match self {
            ConflictPolicy::KeepExisting => current,
            ConflictPolicy::Overwrite => incoming,
            ConflictPolicy::Min => current.min(incoming),
            ConflictPolicy::Max => current.max(incoming),
            ConflictPolicy::Fold(f) => f(&current, &incoming),
        }
    }
}

/// 空間IDコレクションに対して単項演算を行うTrait。
/// 必要な場合は[Self::CustomParameter]に[ConflictPolicy]を含む。
pub trait UnaryOperator<A: CellValue> {
    #[cfg(feature = "rayon")]
    type CustomParameter: Sync;

    #[cfg(not(feature = "rayon"))]
    type CustomParameter;

    /// 結果として帰ってくる値の型
    type ResultValue: CellValue;

    /// コレクションに対する単項演算の定義
    fn execution<S, O>(a: &S, custom_parameter: Self::CustomParameter) -> Result<O, Error>
    where
        S: SpatialIdCollection<Value = A>,
        O: SpatialIdCollection<Value = Self::ResultValue>;

    /// この演算が恒等変換かを判定する。
    fn is_identity(_custom_parameter: &Self::CustomParameter) -> bool;
}

/// `base` から `holes` の各領域を順に差し引いた、残りの領域の集合を返す。
fn subtract_regions(base: FlexId, holes: &[FlexId]) -> Vec<FlexId> {
    let mut regions = vec![base];

    for hole in holes {
        let mut next = Vec::new();
        for region in regions {
            next.extend(region.difference(hole));
        }
        regions = next;
    }

    regions
}
