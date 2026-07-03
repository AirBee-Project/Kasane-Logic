//! 空間主体の時空間集合 [`SpatioTemporalSet`]。
//!
//! # 設計（空間主体）
//! - 空間は既存の [`SpatialIdTable`] ツリーが一次索引（キーは常に temporal=WHOLE の空間セル）。
//! - 時間は各空間セルの **値** [`TemporalSet`]（そのセルが存在する時間）として保持する。
//! - 集合演算はテスト済みの [`BinaryOperator::execution`]（値結合スキャン・空間重なり分解込み）
//!   に、時間結合オペレータ（union/intersection/difference）を差し込んで行う。
//!
//! これにより空間側のスケーラビリティ（構造共有・異方マージ）を保ちつつ、時間を正しく合成する。
//! 空間セルは「時間構造が等しければ」マージされるので、全時間データは従来同様に畳まれる。

use alloc::vec::Vec;

use crate::{BinaryOperator, Error, FlexId, SpatialId, SpatialIdTable, TemporalId, TemporalSet};

// ---- 時間結合オペレータ（値 = TemporalSet） ----

/// 時間の和（both は union、片側はそのまま）。
struct TUnion;
impl BinaryOperator<TemporalSet, TemporalSet> for TUnion {
    type CustomParameter = ();
    type ResultValue = TemporalSet;

    fn both_some(a: &TemporalSet, b: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(Some(a.union(b)))
    }
    fn a_only(a: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(Some(a.clone()))
    }
    fn b_only(b: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(Some(b.clone()))
    }
    fn is_commutative(_: &()) -> bool {
        true
    }
}

/// 時間の積（both は intersection、片側のみは存在しない）。
struct TIntersection;
impl BinaryOperator<TemporalSet, TemporalSet> for TIntersection {
    type CustomParameter = ();
    type ResultValue = TemporalSet;

    fn both_some(a: &TemporalSet, b: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        let i = a.intersection(b);
        Ok(if i.is_empty() { None } else { Some(i) })
    }
    fn a_only(_: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(None)
    }
    fn b_only(_: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(None)
    }
    fn is_commutative(_: &()) -> bool {
        true
    }
}

/// 時間の差（both は difference、a のみはそのまま、b のみは存在しない）。
struct TDifference;
impl BinaryOperator<TemporalSet, TemporalSet> for TDifference {
    type CustomParameter = ();
    type ResultValue = TemporalSet;

    fn both_some(a: &TemporalSet, b: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        let d = a.difference(b);
        Ok(if d.is_empty() { None } else { Some(d) })
    }
    fn a_only(a: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(Some(a.clone()))
    }
    fn b_only(_: &TemporalSet, _: &()) -> Result<Option<TemporalSet>, Error> {
        Ok(None)
    }
    fn is_commutative(_: &()) -> bool {
        false
    }
}

/// FlexId の空間部分だけを取り出す（temporal=WHOLE の空間セル）。ツリーのキーに使う。
fn spatial_cell(f: &FlexId) -> FlexId {
    FlexId::new_with_temporal(
        f.f_zoomlevel(),
        f.f_index(),
        f.x_zoomlevel(),
        f.x_index(),
        f.y_zoomlevel(),
        f.y_index(),
        TemporalId::WHOLE,
    )
    .expect("spatial part is valid")
}

/// 空間セルに時間セルを付けた FlexId を作る。
fn with_temporal(spatial: &FlexId, t: TemporalId) -> FlexId {
    FlexId::new_with_temporal(
        spatial.f_zoomlevel(),
        spatial.f_index(),
        spatial.x_zoomlevel(),
        spatial.x_index(),
        spatial.y_zoomlevel(),
        spatial.y_index(),
        t,
    )
    .expect("spatio-temporal id is valid")
}

/// 空間主体の時空間集合。
#[derive(Clone, Debug, Default)]
pub struct SpatioTemporalSet {
    /// 空間セル（temporal=WHOLE）→ その空間が存在する時間集合。
    inner: SpatialIdTable<TemporalSet>,
}

impl SpatioTemporalSet {
    /// 空の集合を作る。
    pub fn new() -> Self {
        Self {
            inner: SpatialIdTable::new(),
        }
    }

    /// 時空間ID（[`FlexId`] は時間付き可）を挿入する。
    /// 既存と空間が重なる場合は、その領域で時間を union して合成する。
    pub fn insert<S: SpatialId>(&mut self, target: S) {
        let mut single = SpatialIdTable::<TemporalSet>::new();
        for flex_id in target.iter_flex_ids() {
            single.insert(
                spatial_cell(&flex_id),
                TemporalSet::from_temporal(flex_id.temporal()),
            );
        }
        let merged: SpatialIdTable<TemporalSet> =
            TUnion::execution(core::mem::take(&mut self.inner), single, ())
                .expect("temporal union never fails");
        self.inner = merged;
    }

    /// 複数の時空間ID をまとめて構築する（バルク）。
    ///
    /// 逐次 `insert`（毎回 self と union、O(n²)）ではなく、singleton を
    /// **tree-reduce で union** して O(n log n) 回の union に抑える。結果は逐次 insert と一致。
    pub fn from_flex_ids<I: IntoIterator<Item = FlexId>>(iter: I) -> Self {
        let mut level: Vec<Self> = iter
            .into_iter()
            .map(|f| {
                let mut s = Self::new();
                s.insert(f);
                s
            })
            .collect();
        while level.len() > 1 {
            let mut next = Vec::with_capacity(level.len().div_ceil(2));
            let mut it = level.into_iter();
            while let Some(a) = it.next() {
                match it.next() {
                    Some(b) => next.push(a.union(&b)),
                    None => next.push(a),
                }
            }
            level = next;
        }
        level.into_iter().next().unwrap_or_default()
    }

    /// 空かどうか。
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// 保持している全 (空間セル, 時間集合) を、時空間 [`FlexId`] 列として返す。
    pub fn iter(&self) -> impl Iterator<Item = FlexId> + '_ {
        self.inner.iter().flat_map(|(spatial, tset)| {
            tset.cells()
                .into_iter()
                .map(move |t| with_temporal(&spatial, t))
        })
    }

    /// 和集合。
    pub fn union(&self, other: &Self) -> Self {
        Self {
            inner: TUnion::execution(self.inner.clone(), other.inner.clone(), ())
                .expect("temporal union never fails"),
        }
    }

    /// 積集合（空間が重なり、かつ時間も重なる部分）。
    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            inner: TIntersection::execution(self.inner.clone(), other.inner.clone(), ())
                .expect("temporal intersection never fails"),
        }
    }

    /// 差集合。
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            inner: TDifference::execution(self.inner.clone(), other.inner.clone(), ())
                .expect("temporal difference never fails"),
        }
    }

    /// `query`（時空間 [`FlexId`]）と重なる部分を、`query` に切り取って返す（時空間クエリ）。
    ///
    /// 空間で交差した領域ごとに、時間も `query` の時間と交差させ、残った時空間セルを返す。
    /// UTM の衝突判定（「同じ空間 かつ 時間も重なる」）の中核。
    pub fn get(&self, query: &FlexId) -> impl Iterator<Item = FlexId> {
        let q_spatial = spatial_cell(query);
        let q_time = TemporalSet::from_temporal(query.temporal());
        // 空間交差（FlexId は交差領域・temporal=WHOLE）と、その時間集合を取り出す。
        let hits: Vec<(FlexId, TemporalSet)> = self
            .inner
            .get(&q_spatial)
            .map(|(f, tset)| (f, tset.clone()))
            .collect();

        let mut out = Vec::new();
        for (spatial_inter, tset) in hits {
            let overlap = tset.intersection(&q_time);
            for t in overlap.cells() {
                out.push(with_temporal(&spatial_inter, t));
            }
        }
        out.into_iter()
    }

    /// `query` と重なる部分を切り抜いて削除し、削除した時空間セルを返す。
    pub fn remove(&mut self, query: &FlexId) -> impl Iterator<Item = FlexId> {
        let mut q = SpatioTemporalSet::new();
        q.insert(query.clone());
        let removed: Vec<FlexId> = self.intersection(&q).iter().collect();
        *self = self.difference(&q);
        removed.into_iter()
    }

    /// 各空間セルの時間を `window` に切り詰めた集合を返す（時間窓で限定）。
    pub fn clip_time(&self, window: &TemporalId) -> Self {
        let w = TemporalSet::from_temporal(window);
        let mut out = SpatialIdTable::<TemporalSet>::new();
        for (spatial, tset) in self.inner.iter() {
            let clipped = tset.intersection(&w);
            if !clipped.is_empty() {
                out.insert(spatial, clipped);
            }
        }
        Self { inner: out }
    }

    /// 時間窓 `window` に限定した差集合 `(self ∩ window) − other`（方法2）。
    ///
    /// `self` の時間が WHOLE でも `window` が有界なら結果は有界になる
    /// （`(A ∩ W) − B = (A − B) ∩ W`）。
    pub fn difference_in_window(&self, other: &Self, window: &TemporalId) -> Self {
        self.clip_time(window).difference(other)
    }
}

#[cfg(test)]
mod tests {
    use super::SpatioTemporalSet;
    use crate::{FlexId, SpatialId, TemporalId};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    type Atom = ((i32, u32, u32), u64);

    /// FlexId の空間部分を共通ズーム z の (f,x,y) 群へ展開。
    fn spatial_keys(f: &FlexId, z: u8) -> Vec<(i32, u32, u32)> {
        let (fz, xz, yz) = (f.f_zoomlevel(), f.x_zoomlevel(), f.y_zoomlevel());
        let f0 = f.f_index() << (z - fz);
        let x0 = f.x_index() << (z - xz);
        let y0 = f.y_index() << (z - yz);
        let (fs, xs, ys) = (1i32 << (z - fz), 1u32 << (z - xz), 1u32 << (z - yz));
        let mut out = Vec::new();
        for df in 0..fs {
            for dx in 0..xs {
                for dy in 0..ys {
                    out.push((f0 + df, x0 + dx, y0 + dy));
                }
            }
        }
        out
    }

    /// 時空間集合を (空間キー × 秒) のアトム集合へ展開（オラクル）。
    fn atoms(set: &SpatioTemporalSet, z: u8) -> BTreeSet<Atom> {
        let mut out = BTreeSet::new();
        for f in set.iter() {
            let secs: Vec<u64> = (f.temporal().start_unixstamp()
                ..f.temporal().end_unixtime_exclusive() as u64)
                .collect();
            for k in spatial_keys(&f, z) {
                for &s in &secs {
                    out.insert((k, s));
                }
            }
        }
        out
    }

    /// 時間付き FlexId を作る（zoom, f=x=y、時間セル (i,t)）。
    fn cell(z: u8, f: i32, x: u32, y: u32, i: u64, t: u64) -> FlexId {
        FlexId::new_with_temporal(z, f, z, x, z, y, TemporalId::new(i, t).unwrap()).unwrap()
    }

    fn build(cells: &[FlexId]) -> SpatioTemporalSet {
        let mut s = SpatioTemporalSet::new();
        for c in cells {
            s.insert(c.clone());
        }
        s
    }

    /// insert が空間重なりで時間 union するか（同一空間セルに別時間 → 両方保持）。
    #[test]
    fn insert_merges_temporal_at_same_cell() {
        let mut s = SpatioTemporalSet::new();
        s.insert(cell(2, 0, 0, 0, 60, 0)); // (2,0,0,0) @ [0,60)
        s.insert(cell(2, 0, 0, 0, 60, 2)); // 同じ空間 @ [120,180)
        let a = atoms(&s, 2);
        let exp: BTreeSet<Atom> = (0..60u64)
            .chain(120..180)
            .map(|sec| ((0, 0, 0), sec))
            .collect();
        assert_eq!(a, exp);
    }

    /// union / intersection / difference をアトムオラクルで厳密照合。
    #[test]
    fn set_ops_atom_oracle() {
        // A: (2,0,0,0)@[0,3600)  と (2,0,1,0)@[0,60)
        let a = build(&[cell(2, 0, 0, 0, 3600, 0), cell(2, 0, 1, 0, 60, 0)]);
        // B: (2,0,0,0)@[0,60)（Aの部分空間×部分時間） と (2,0,2,0)@[0,60)（Aに無い空間）
        let b = build(&[cell(2, 0, 0, 0, 60, 0), cell(2, 0, 2, 0, 60, 0)]);

        let (aa, ba) = (atoms(&a, 2), atoms(&b, 2));

        assert_eq!(atoms(&a.union(&b), 2), aa.union(&ba).copied().collect());
        assert_eq!(
            atoms(&a.intersection(&b), 2),
            aa.intersection(&ba).copied().collect()
        );
        assert_eq!(
            atoms(&a.difference(&b), 2),
            aa.difference(&ba).copied().collect()
        );
    }

    /// get（時空間クエリ）: 結果アトム == 集合アトム ∩ クエリアトム。
    #[test]
    fn get_atom_oracle() {
        let a = build(&[cell(2, 0, 0, 0, 3600, 0), cell(2, 0, 1, 0, 60, 0)]);
        // クエリ: (1,0,0,0)@[0,120)  … 空間は (2,0,0,0)/(2,0,1,0) を含む粗いセル
        let query =
            FlexId::new_with_temporal(1u8, 0, 1u8, 0, 1u8, 0, TemporalId::new(60, 1).unwrap())
                .unwrap(); // [60,120)
        let got: BTreeSet<Atom> = {
            let mut out = BTreeSet::new();
            for f in a.get(&query) {
                let secs: Vec<u64> = (f.temporal().start_unixstamp()
                    ..f.temporal().end_unixtime_exclusive() as u64)
                    .collect();
                for k in spatial_keys(&f, 2) {
                    for &s in &secs {
                        out.insert((k, s));
                    }
                }
            }
            out
        };
        // オラクル: atoms(a) のうち、query の (空間×時間) に入るもの
        let qa: BTreeSet<Atom> = spatial_keys(&query, 2)
            .into_iter()
            .flat_map(|k| (60u64..120).map(move |s| (k, s)))
            .collect();
        let exp: BTreeSet<Atom> = atoms(&a, 2).intersection(&qa).copied().collect();
        assert_eq!(got, exp);
    }

    /// remove: 削除アトム ＝ 元 ∩ クエリ、残り ＝ 元 − クエリ。
    #[test]
    fn remove_atom_oracle() {
        let mut a = build(&[cell(2, 0, 0, 0, 3600, 0)]);
        let before = atoms(&a, 2);
        let query = cell(2, 0, 0, 0, 60, 0); // [0,60)
        let removed: BTreeSet<Atom> = {
            let mut out = BTreeSet::new();
            for f in a.remove(&query) {
                let secs: Vec<u64> = (f.temporal().start_unixstamp()
                    ..f.temporal().end_unixtime_exclusive() as u64)
                    .collect();
                for k in spatial_keys(&f, 2) {
                    for &s in &secs {
                        out.insert((k, s));
                    }
                }
            }
            out
        };
        let qa: BTreeSet<Atom> = (0u64..60).map(|s| ((0, 0, 0), s)).collect();
        assert_eq!(removed, before.intersection(&qa).copied().collect());
        assert_eq!(atoms(&a, 2), before.difference(&qa).copied().collect());
    }

    /// difference_in_window: WHOLE 起点の差分を窓で有界化しつつ正しい。
    #[test]
    fn difference_in_window_bounds_whole() {
        // A: (2,0,0,0) @ WHOLE
        let mut a = SpatioTemporalSet::new();
        a.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap());
        // B: (2,0,0,0) @ [0,60)
        let b = build(&[cell(2, 0, 0, 0, 60, 0)]);
        // 窓: その1時間 [0,3600)
        let window = TemporalId::new(3600, 0).unwrap();
        let d = a.difference_in_window(&b, &window);
        // オラクル: (WHOLE − [0,60)) ∩ [0,3600) = [60,3600)、空間 (2,0,0,0)
        let got = atoms(&d, 2);
        let exp: BTreeSet<Atom> = (60u64..3600).map(|s| ((0, 0, 0), s)).collect();
        assert_eq!(got, exp);
    }

    /// バルク from_flex_ids が逐次 insert と一致する。
    #[test]
    fn bulk_matches_sequential() {
        let cells = [
            cell(2, 0, 0, 0, 3600, 0),
            cell(2, 0, 0, 0, 60, 61), // 同じ空間・別時間 → union
            cell(2, 0, 1, 0, 60, 0),
            cell(2, 0, 2, 0, 1, 5),
        ];
        let seq = build(&cells);
        let bulk = SpatioTemporalSet::from_flex_ids(cells.iter().cloned());
        assert_eq!(atoms(&seq, 2), atoms(&bulk, 2));
    }

    /// 空間のみ（時間 WHOLE）でも動く（時間 WHOLE 同士の union/diff）。
    #[test]
    fn spatial_only_whole() {
        let mut a = SpatioTemporalSet::new();
        a.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap()); // WHOLE
        let mut b = SpatioTemporalSet::new();
        b.insert(FlexId::new(2, 0, 2, 0, 2, 0).unwrap()); // 同じ、WHOLE
        // union は同じ、difference は空
        assert!(!a.union(&b).is_empty());
        assert!(a.difference(&b).is_empty());
    }
}
