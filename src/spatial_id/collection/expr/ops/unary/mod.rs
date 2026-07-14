pub mod kernel;
pub mod op;

pub use kernel::*;
pub use op::*;

/// 特定の次元の方向へ移動
pub mod shift;

/// 特定の次元の方向へ引き延ばし
pub mod stretch;

/// X / Y 平面上で値を同心円状に伝播（減衰）
pub mod spread;

/// 特定の次元の占有を絶対座標範囲へ揃える（起伏を平坦化）
pub mod level;

/// 値を持つ領域の最小範囲（AABB）の隙間へ既定値を割り当てる
pub mod fill;

use alloc::vec::Vec;

use crate::{CellValue, Error, FlexId};

/// 各セル `(FlexId, 値)` へ `transform`（幾何変換）を適用し、変換後の
/// `(出力 FlexId, 値)` のリストへ平坦化する。
pub(crate) fn map_cells<I, A, F>(cells: I, transform: F) -> Result<Vec<(FlexId, A)>, Error>
where
    I: IntoIterator<Item = (FlexId, A)>,
    A: CellValue,
    F: Fn(&FlexId) -> Result<Vec<FlexId>, Error> + Send + Sync,
{
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        let cells_vec: Vec<_> = cells.into_iter().collect();
        cells_vec
            .into_par_iter()
            .map(|(id, value)| {
                let outputs = transform(&id)?;
                Ok(outputs
                    .into_iter()
                    .map(move |o| (o, value.clone()))
                    .collect::<Vec<_>>())
            })
            .collect::<Result<Vec<Vec<_>>, Error>>()
            .map(|grouped| grouped.into_iter().flatten().collect())
    }

    #[cfg(not(feature = "rayon"))]
    {
        let mut out = Vec::new();
        for (id, value) in cells.into_iter() {
            for o in transform(&id)? {
                out.push((o, value.clone()));
            }
        }
        Ok(out)
    }
}
