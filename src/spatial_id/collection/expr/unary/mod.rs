/// 特定の次元の方向へ移動
pub mod shift;

/// 特定の次元の方向へ引き延ばし
pub mod stretch;

/// 特定の次元の占有を絶対座標範囲へ揃える（起伏を平坦化）
pub mod level;

/// 値を持つ領域の最小範囲（AABB）の隙間へ既定値を割り当てる
pub mod fill;

use alloc::vec::Vec;

use crate::{CellValue, Error, FlexId};

/// 各セル `(FlexId, 値)` へ `transform`（幾何変換）を適用し、変換後の
/// `(出力 FlexId, 値)` のリストへ平坦化する。
///
/// `transform` は純粋な変換のみを担い、`result` への書き込み（重なり解決を含む）は
/// 呼び出し側が変換結果を**元の順序のまま逐次** insert する。Rayon の `collect` は
/// 順序を保つため、並列にしても挿入順＝逐次版と一致し、結果は変わらない。
///
/// `rayon` feature が有効で入力が十分大きいときのみ並列実行し、それ以外は逐次。
pub(crate) fn map_cells<A, F>(
    cells: Vec<(FlexId, A)>,
    transform: F,
) -> Result<Vec<(FlexId, A)>, Error>
where
    A: CellValue,
    F: Fn(&FlexId) -> Result<Vec<FlexId>, Error> + Send + Sync,
{
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;

        // 並列化のオーバーヘッドが見合う規模でのみ並列化する。
        const PAR_THRESHOLD: usize = 256;
        if cells.len() >= PAR_THRESHOLD {
            return cells
                .into_par_iter()
                .map(|(id, value)| {
                    let outputs = transform(&id)?;
                    Ok(outputs.into_iter().map(move |o| (o, value.clone())).collect::<Vec<_>>())
                })
                .collect::<Result<Vec<Vec<_>>, Error>>()
                .map(|grouped| grouped.into_iter().flatten().collect());
        }
    }

    // 逐次（rayon 無効、または小入力）。
    let mut out = Vec::new();
    for (id, value) in cells {
        for o in transform(&id)? {
            out.push((o, value.clone()));
        }
    }
    Ok(out)
}
