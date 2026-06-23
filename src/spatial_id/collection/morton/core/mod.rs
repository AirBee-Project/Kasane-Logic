//! Morton order バックエンドの中核データ構造。
//!
//! 各セルを [`SingleId::spatial_encode`] による 12 バイトの Morton キーへ符号化し、
//! `BTreeMap<MortonKey, V>` に格納する（古典的 linear quadtree 表現）。FlexTree のような
//! 構造共有・領域圧縮は行わず、単一解像度セルのフラットな順序付き集合として保持する。
//!
//! 階層グリッドのセルは「互いに素」か「入れ子」のいずれかである性質を使い、
//! 入れ子（祖先・子孫）を正しく扱う。マップは常に「どのセルも他セルの祖先/子孫でない」
//! 反鎖（antichain）として正規化される。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::{FlexId, IntoSingleIds, IterFlexIds, RangeId, SingleId, SpatialId};

mod ops;

/// セルを表す 12 バイトの Morton キー。
pub type MortonKey = [u8; 12];

/// マップ内の全セルのズームレベルの状態。集合演算の高速経路判定に使う。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoomState {
    /// 空。
    Empty,
    /// 全セルが同一ズームレベル `z`。集合演算を純粋なキー集合演算へ落とせる。
    Uniform(u8),
    /// 複数のズームレベルが混在。一般経路（入れ子考慮）が必要。
    Mixed,
}

impl ZoomState {
    fn add(self, z: u8) -> Self {
        match self {
            ZoomState::Empty => ZoomState::Uniform(z),
            ZoomState::Uniform(zz) if zz == z => ZoomState::Uniform(z),
            _ => ZoomState::Mixed,
        }
    }
}

/// Morton キーを `u128`（下位 96 ビットにキーを配置）へ展開する。
#[inline]
fn key_to_u128(k: &MortonKey) -> u128 {
    let mut b = [0u8; 16];
    b[4..].copy_from_slice(k);
    u128::from_be_bytes(b)
}

/// `u128` の下位 96 ビットを Morton キーへ戻す。
#[inline]
fn u128_to_key(v: u128) -> MortonKey {
    let b = v.to_be_bytes();
    let mut k = [0u8; 12];
    k.copy_from_slice(&b[4..]);
    k
}

/// キーに埋め込まれたズームレベル（最下位 5 ビット）を取り出す。
#[inline]
pub fn key_zoom(k: &MortonKey) -> u8 {
    k[11] & 0b1_1111
}

/// キーから [`SingleId`] を復元する。格納済みキーは常に妥当なので失敗しない。
#[inline]
pub fn decode(k: &MortonKey) -> SingleId {
    SingleId::spatial_decode(k).expect("stored morton key must decode")
}

/// `sid` の「子孫または自身」が取り得るキー範囲 `[lo, hi]`（両端含む）を返す。
///
/// パス部の上位 `3*z+1` ビットを固定し、それより下位の全ビット（ズーム欄を含む）を
/// `0` にしたものを下限、`1` にしたものを上限とする。これにより子孫のズームレベルに
/// 依らず全子孫を確実に囲む。
fn descendant_bounds(sid: &SingleId) -> (MortonKey, MortonKey) {
    let base = key_to_u128(&sid.spatial_encode());
    let path_bits = 3u32 * sid.z() as u32 + 1;
    let free = 96u32.saturating_sub(path_bits);
    let mask: u128 = if free == 0 { 0 } else { (1u128 << free) - 1 };
    let full96 = (1u128 << 96) - 1;
    let lo = base & !mask & full96;
    let hi = base | mask;
    (u128_to_key(lo), u128_to_key(hi))
}

/// 拡張空間IDとそれに紐づいた値を Morton order で保持する型。
#[derive(Clone, Debug)]
pub struct MortonCore<V> {
    pub(crate) cells: BTreeMap<MortonKey, V>,
    pub(crate) zoom_state: ZoomState,
}

impl<V> Default for MortonCore<V>
where
    V: Clone + Ord + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> PartialEq for MortonCore<V>
where
    V: Clone + Ord + Send + Sync,
{
    fn eq(&self, other: &Self) -> bool {
        self.cells == other.cells
    }
}
impl<V> Eq for MortonCore<V> where V: Clone + Ord + Send + Sync {}

impl<V> MortonCore<V>
where
    V: Clone + Ord + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            cells: BTreeMap::new(),
            zoom_state: ZoomState::Empty,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn count(&self) -> usize {
        self.cells.len()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.zoom_state = ZoomState::Empty;
    }

    pub fn max_zoomlevel(&self) -> Option<u8> {
        self.cells.keys().map(key_zoom).max()
    }

    /// マップ内の最小ズームレベル（祖先方向の正規化補助）。
    fn min_zoomlevel(&self) -> Option<u8> {
        self.cells.keys().map(key_zoom).min()
    }

    /// `zoom_state` をマップ全体から再計算する（一般経路の構築後に使う）。
    fn recompute_zoom_state(&mut self) {
        let mut state = ZoomState::Empty;
        for k in self.cells.keys() {
            state = state.add(key_zoom(k));
        }
        self.zoom_state = state;
    }

    /// `sid`（自身）またはその祖先で、マップに格納済みのセルがあればその参照を返す。
    /// 反鎖不変条件より、該当は高々 1 つ。
    pub(crate) fn covering(&self, sid: &SingleId) -> Option<(SingleId, &V)> {
        let min_z = self.min_zoomlevel()?;
        // 自身 → 祖先の順に、格納され得るズームだけを点検索する。
        for z in (min_z..=sid.z()).rev() {
            let anc = sid
                .spatial_parent_at_zoom(z)
                .expect("z <= sid.z must be valid");
            if let Some(v) = self.cells.get(&anc.spatial_encode()) {
                return Some((anc, v));
            }
        }
        None
    }

    /// `sid` の厳密な子孫（自身を除く）で格納済みのセルを取り除く。
    fn remove_strict_descendants(&mut self, sid: &SingleId) {
        let self_key = sid.spatial_encode();
        let (lo, hi) = descendant_bounds(sid);
        let doomed: Vec<MortonKey> = self
            .cells
            .range(lo..=hi)
            .map(|(k, _)| *k)
            .filter(|k| *k != self_key)
            .collect();
        for k in doomed {
            self.cells.remove(&k);
        }
    }

    /// 1 つの [`SingleId`] セルへ値を書き込む（FlexTree の上書き挿入に相当）。
    ///
    /// - 祖先が同領域を覆っている場合は祖先を `sid` の解像度へ分割し、`sid` 以外は旧値を保つ。
    /// - 子孫が格納されている場合は（上書きされるため）取り除く。
    pub fn insert_cell(&mut self, sid: SingleId, value: V) {
        let self_key = sid.spatial_encode();

        // すでに同一セルがあるなら値だけ更新（子孫は反鎖よりそもそも無い）。
        if let Some(slot) = self.cells.get_mut(&self_key) {
            *slot = value;
            return;
        }

        // 祖先が覆っているか確認。
        if let Some(min_z) = self.min_zoomlevel() {
            for z in (min_z..sid.z()).rev() {
                let anc = sid
                    .spatial_parent_at_zoom(z)
                    .expect("z < sid.z must be valid");
                let anc_key = anc.spatial_encode();
                if let Some(old) = self.cells.get(&anc_key).cloned() {
                    // 祖先を sid の解像度へ分割し、sid 以外を旧値で復元する。
                    self.cells.remove(&anc_key);
                    for child in anc
                        .spatial_children_at_zoom(sid.z())
                        .expect("sid.z >= anc.z")
                    {
                        if child != sid {
                            self.cells.insert(child.spatial_encode(), old.clone());
                        }
                    }
                    self.cells.insert(self_key, value);
                    // 分割により子解像度のセルが混ざるので状態を更新。
                    self.zoom_state = self.zoom_state.add(sid.z());
                    return;
                }
            }
        }

        // 祖先なし：子孫を掃除して挿入。
        self.remove_strict_descendants(&sid);
        self.cells.insert(self_key, value);
        self.zoom_state = self.zoom_state.add(sid.z());
    }

    /// [`IterFlexIds`] な対象を単一解像度セルへ展開して挿入する。
    pub fn insert<S: IterFlexIds>(&mut self, target: S, value: V) {
        for flex_id in target.iter_flex_ids() {
            if cfg!(not(feature = "temporal_id")) && !flex_id.temporal().is_whole() {
                panic!("TemporalIdはMortonバックエンドに挿入できません。");
            }
            for single_id in RangeId::from(&flex_id).into_single_ids() {
                self.insert_cell(single_id, value.clone());
            }
        }
    }

    /// 単一の [`SingleId`] クエリに重なる格納済みセル（クリップ後）を列挙する。
    ///
    /// 入れ子性より、結果は「`q` を覆う唯一の祖先（→ `q` にクリップ）」または
    /// 「`q` 配下の子孫セル群」のいずれかになる。
    pub(crate) fn query_single(&self, q: &SingleId) -> Vec<(SingleId, V)> {
        // 祖先または自身が覆う場合は q にクリップして返す。
        if let Some((_, v)) = self.covering(q) {
            return alloc::vec![(q.clone(), v.clone())];
        }
        // そうでなければ q 配下の子孫セルを返す。
        let self_key = q.spatial_encode();
        let (lo, hi) = descendant_bounds(q);
        self.cells
            .range(lo..=hi)
            .filter(|(k, _)| **k != self_key)
            .map(|(k, v)| (decode(k), v.clone()))
            .collect()
    }

    /// 対象と重なる `(FlexId, V)` を、重なり部分へクリップして返す。
    pub fn get<'a, S>(&'a self, target: &'a S) -> impl Iterator<Item = (FlexId, V)> + 'a
    where
        S: IterFlexIds + 'a,
    {
        let mut seen = hashbrown::HashSet::new();
        let mut out = Vec::new();
        for flex_id in target.iter_flex_ids() {
            for single in RangeId::from(&flex_id).into_single_ids() {
                for (sid, v) in self.query_single(&single) {
                    let fid = FlexId::from(&sid);
                    if seen.insert(fid.clone()) {
                        out.push((fid, v));
                    }
                }
            }
        }
        out.into_iter()
    }

    /// 対象が示す領域を取り除き、取り除いたセルを返す。
    pub fn remove<S: IterFlexIds>(&mut self, target: &S) -> Vec<(SingleId, V)> {
        let mut removed = Vec::new();
        for flex_id in target.iter_flex_ids() {
            for single in RangeId::from(&flex_id).into_single_ids() {
                for (sid, v) in self.query_single(&single) {
                    // 祖先が覆っている場合は分割が必要なので insert ベースで処理する。
                    self.cut_cell(&sid);
                    removed.push((sid, v));
                }
            }
        }
        if !removed.is_empty() {
            self.recompute_zoom_state();
        }
        removed
    }

    /// ちょうど `sid` の領域だけをマップから除去する（祖先は分割、子孫は削除）。
    fn cut_cell(&mut self, sid: &SingleId) {
        let self_key = sid.spatial_encode();
        if self.cells.remove(&self_key).is_some() {
            return;
        }
        // 祖先が覆うなら分割して sid 部分だけ空ける。
        if let Some(min_z) = self.min_zoomlevel() {
            for z in (min_z..sid.z()).rev() {
                let anc = sid.spatial_parent_at_zoom(z).expect("valid");
                let anc_key = anc.spatial_encode();
                if let Some(old) = self.cells.remove(&anc_key) {
                    for child in anc.spatial_children_at_zoom(sid.z()).expect("valid") {
                        if &child != sid {
                            self.cells.insert(child.spatial_encode(), old.clone());
                        }
                    }
                    return;
                }
            }
        }
        // 子孫のみ存在する場合は削除。
        self.remove_strict_descendants(sid);
    }

    /// 全セルを [`SingleId`] として走査する。
    pub fn iter_single(&self) -> impl Iterator<Item = (SingleId, &V)> + '_ {
        self.cells.iter().map(|(k, v)| (decode(k), v))
    }

    /// 全セルを `(FlexId, V)` として走査する（クローン）。
    pub fn iter(&self) -> impl Iterator<Item = (FlexId, V)> + '_ {
        self.cells
            .iter()
            .map(|(k, v)| (FlexId::from(&decode(k)), v.clone()))
    }

    /// 全セルを `(FlexId, &V)` として走査する（参照）。
    pub fn iter_ref(&self) -> impl Iterator<Item = (FlexId, &V)> + '_ {
        self.cells
            .iter()
            .map(|(k, v)| (FlexId::from(&decode(k)), v))
    }

    /// 値が存在する全セルを包む最小の [`RangeId`]（3次元AABB）。
    pub fn bounding_box(&self) -> Option<RangeId> {
        RangeId::bounding_box_of(self.iter().map(|(flex_id, _)| flex_id))
    }

    /// [`get`](Self::get) の [`SingleId`] 版（重なり部分へクリップ）。
    pub fn get_single<'a, S>(&'a self, target: &'a S) -> Vec<(SingleId, V)>
    where
        S: IterFlexIds + 'a,
    {
        let mut seen = hashbrown::HashSet::new();
        let mut out = Vec::new();
        for flex_id in target.iter_flex_ids() {
            for single in RangeId::from(&flex_id).into_single_ids() {
                for (sid, v) in self.query_single(&single) {
                    if seen.insert(sid.spatial_encode()) {
                        out.push((sid, v));
                    }
                }
            }
        }
        out
    }

    /// 単一クエリに重なる格納済みセルを **クリップせず** に列挙する。
    fn overlapping_single(&self, q: &SingleId) -> Vec<(SingleId, V)> {
        if let Some((anc, v)) = self.covering(q) {
            return alloc::vec![(anc, v.clone())];
        }
        let self_key = q.spatial_encode();
        let (lo, hi) = descendant_bounds(q);
        self.cells
            .range(lo..=hi)
            .filter(|(k, _)| **k != self_key)
            .map(|(k, v)| (decode(k), v.clone()))
            .collect()
    }

    /// 対象と重なった格納済みセルを **そのままの広さ** で（クリップ・重複除去して）返す。
    pub fn get_overlapping_single<S>(&self, target: &S) -> Vec<(SingleId, V)>
    where
        S: IterFlexIds,
    {
        let mut seen = hashbrown::HashSet::new();
        let mut out = Vec::new();
        for flex_id in target.iter_flex_ids() {
            for single in RangeId::from(&flex_id).into_single_ids() {
                for (sid, v) in self.overlapping_single(&single) {
                    if seen.insert(sid.spatial_encode()) {
                        out.push((sid, v));
                    }
                }
            }
        }
        out
    }

    /// 対象と重なった格納済みセルを丸ごと取り除き、そのまま返す。
    pub fn remove_overlapping<S>(&mut self, target: &S) -> Vec<(SingleId, V)>
    where
        S: IterFlexIds,
    {
        let removed = self.get_overlapping_single(target);
        for (sid, _) in &removed {
            self.cells.remove(&sid.spatial_encode());
        }
        if !removed.is_empty() {
            self.recompute_zoom_state();
        }
        removed
    }

    /// 入力 ID と面で接する格納済みセルを重複なく返す（自身と重なる要素は除外）。
    pub fn neighbors_share_face_single<S: SpatialId>(&self, id: &S) -> Vec<(SingleId, V)> {
        let self_cells: Vec<SingleId> = id.iter_single_ids().collect();
        let mut seen = hashbrown::HashSet::new();
        let mut out = Vec::new();

        for s in &self_cells {
            for neighbor in s.neighbors_share_face() {
                for (cand, v) in self.overlapping_single(&neighbor) {
                    // 自身と重なる候補は除外する。
                    if self_cells.iter().any(|sc| cells_overlap(sc, &cand)) {
                        continue;
                    }
                    if seen.insert(cand.spatial_encode()) {
                        out.push((cand, v));
                    }
                }
            }
        }
        out
    }

    /// このコアを、全体の最大ズームへ揃えた [`SingleId`] 列として書き出す。
    pub fn flat_single_ids(&self) -> Vec<(SingleId, V)> {
        let Some(max_z) = self.max_zoomlevel() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (sid, v) in self.iter_single() {
            if sid.z() == max_z {
                out.push((sid, v.clone()));
            } else {
                for child in sid.spatial_children_at_zoom(max_z).expect("max_z >= sid.z") {
                    out.push((child, v.clone()));
                }
            }
        }
        out
    }
}

/// 2つのセルが空間的に重なる（同一・祖先・子孫のいずれか）かを判定する。
fn cells_overlap(a: &SingleId, b: &SingleId) -> bool {
    let (coarse, fine) = if a.z() <= b.z() { (a, b) } else { (b, a) };
    fine.spatial_parent_at_zoom(coarse.z())
        .map(|p| &p == coarse)
        .unwrap_or(false)
}
