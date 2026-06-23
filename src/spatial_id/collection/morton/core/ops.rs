//! Morton コアの集合演算（和・積・差）。
//!
//! 全セルが同一ズームレベルのときは純粋なキー集合演算（ソート済みマージ）へ落とす
//! 高速経路を持つ。ズーム混在時は入れ子を考慮した一般経路へ分岐し、`rayon` 有効時は
//! 各セルの寄与計算を並列化する。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::SingleId;

use super::{MortonCore, MortonKey, ZoomState, decode, descendant_bounds};

#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// 部分木が十分大きいときだけ並列化する閾値（タスク生成コストの回避）。
#[cfg(feature = "rayon")]
const PARALLEL_CELL_CUTOFF: usize = 1024;

impl<V> MortonCore<V>
where
    V: Clone + Ord + Send + Sync,
{
    /// 互いに素なセル列から構築する（正規化を省略する高速版）。
    fn from_cells_disjoint(cells: Vec<(MortonKey, V)>) -> Self {
        let mut map = BTreeMap::new();
        let mut state = ZoomState::Empty;
        let mut min_zoom: Option<u8> = None;
        for (k, v) in cells {
            let z = super::key_zoom(&k);
            state = state.add(z);
            min_zoom = Some(min_zoom.map_or(z, |m: u8| m.min(z)));
            map.insert(k, v);
        }
        Self {
            cells: map,
            zoom_state: state,
            min_zoom,
        }
    }

    /// `self` の各セルへ並列に `f` を適用し、寄与（互いに素）を集めて結果を作る。
    fn par_map_cells<F>(&self, f: F) -> Self
    where
        F: Fn(&SingleId, &V) -> Vec<(MortonKey, V)> + Sync + Send,
    {
        let entries: Vec<(SingleId, &V)> = self.cells.iter().map(|(k, v)| (decode(k), v)).collect();

        #[cfg(feature = "rayon")]
        let collected: Vec<(MortonKey, V)> = if entries.len() >= PARALLEL_CELL_CUTOFF {
            entries
                .par_iter()
                .flat_map_iter(|(sid, v)| f(sid, v))
                .collect()
        } else {
            entries.iter().flat_map(|(sid, v)| f(sid, v)).collect()
        };

        #[cfg(not(feature = "rayon"))]
        let collected: Vec<(MortonKey, V)> =
            entries.iter().flat_map(|(sid, v)| f(sid, v)).collect();

        Self::from_cells_disjoint(collected)
    }

    /// `sid` 配下にある `self` の格納済み子孫セルを列挙する。
    fn descendants_under(&self, sid: &SingleId) -> Vec<SingleId> {
        let self_key = sid.spatial_encode();
        let (lo, hi) = descendant_bounds(sid);
        self.cells
            .range(lo..=hi)
            .map(|(k, _)| *k)
            .filter(|k| *k != self_key)
            .map(|k| decode(&k))
            .collect()
    }

    /// 2つのコアの和集合。
    pub fn union(&self, other: &Self) -> Self {
        // 高速経路：同一ズームなら純粋なキー和集合。
        if let (ZoomState::Uniform(za), ZoomState::Uniform(zb)) =
            (self.zoom_state, other.zoom_state)
            && za == zb
        {
            let mut cells = self.cells.clone();
            for (k, v) in &other.cells {
                cells.entry(*k).or_insert_with(|| v.clone());
            }
            return Self {
                cells,
                zoom_state: ZoomState::Uniform(za),
                min_zoom: Some(za),
            };
        }
        if other.is_empty() {
            return self.clone();
        }
        if self.is_empty() {
            return other.clone();
        }

        // 一般経路：self を土台に other のセルを正規化挿入する。
        let mut result = self.clone();
        for (sid, v) in other.iter_single() {
            result.insert_cell(sid, v.clone());
        }
        result
    }

    /// 2つのコアの積集合。
    pub fn intersection(&self, other: &Self) -> Self {
        if let (ZoomState::Uniform(za), ZoomState::Uniform(zb)) =
            (self.zoom_state, other.zoom_state)
            && za == zb
        {
            // 高速経路：両方に存在するキー。小さい側を走査する。
            let (small, large) = if self.cells.len() <= other.cells.len() {
                (&self.cells, &other.cells)
            } else {
                (&other.cells, &self.cells)
            };
            let mut cells = BTreeMap::new();
            for (k, v) in small {
                if large.contains_key(k) {
                    cells.insert(*k, v.clone());
                }
            }
            let (state, min_zoom) = if cells.is_empty() {
                (ZoomState::Empty, None)
            } else {
                (ZoomState::Uniform(za), Some(za))
            };
            return Self {
                cells,
                zoom_state: state,
                min_zoom,
            };
        }

        // 一般経路：self の各セルについて other に覆われた部分を採る。
        self.par_map_cells(|sid, v| {
            if other.covering(sid).is_some() {
                // other の祖先/自身が覆う → セル全体が積。
                alloc::vec![(sid.spatial_encode(), v.clone())]
            } else {
                // other 側の子孫セル（より細かい重なり）を採る。
                other
                    .descendants_under(sid)
                    .into_iter()
                    .map(|d| (d.spatial_encode(), v.clone()))
                    .collect()
            }
        })
    }

    /// 2つのコアの差集合（`self` から `other` の領域を除く）。
    pub fn difference(&self, other: &Self) -> Self {
        if let (ZoomState::Uniform(za), ZoomState::Uniform(zb)) =
            (self.zoom_state, other.zoom_state)
            && za == zb
        {
            // 高速経路：other に無い self のキー。
            let mut cells = BTreeMap::new();
            for (k, v) in &self.cells {
                if !other.cells.contains_key(k) {
                    cells.insert(*k, v.clone());
                }
            }
            let (state, min_zoom) = if cells.is_empty() {
                (ZoomState::Empty, None)
            } else {
                (ZoomState::Uniform(za), Some(za))
            };
            return Self {
                cells,
                zoom_state: state,
                min_zoom,
            };
        }

        // 一般経路：self の各セルから other を引く。
        self.par_map_cells(|sid, v| {
            if other.covering(sid).is_some() {
                // 完全に覆われている → 何も残らない。
                Vec::new()
            } else {
                let holes = other.descendants_under(sid);
                if holes.is_empty() {
                    alloc::vec![(sid.spatial_encode(), v.clone())]
                } else {
                    subtract_holes(sid, &holes)
                        .into_iter()
                        .map(|r| (r.spatial_encode(), v.clone()))
                        .collect()
                }
            }
        })
    }
}

/// `base` から、その配下の細かいセル群 `holes` を引いた残余セルを返す。
///
/// `holes` の最大ズームへ `base` を展開し、穴に覆われない子セルだけを残す。
fn subtract_holes(base: &SingleId, holes: &[SingleId]) -> Vec<SingleId> {
    let target_z = holes.iter().map(|h| h.z()).max().unwrap_or(base.z());

    // 穴が覆うキー集合（target_z 解像度）。
    let mut covered: hashbrown::HashSet<MortonKey> = hashbrown::HashSet::new();
    for h in holes {
        if h.z() == target_z {
            covered.insert(h.spatial_encode());
        } else {
            for child in h
                .spatial_children_at_zoom(target_z)
                .expect("target_z >= h.z")
            {
                covered.insert(child.spatial_encode());
            }
        }
    }

    base.spatial_children_at_zoom(target_z)
        .expect("target_z >= base.z")
        .filter(|child| !covered.contains(&child.spatial_encode()))
        .collect()
}
