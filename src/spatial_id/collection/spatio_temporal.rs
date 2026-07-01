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
