use alloc::vec::Vec;

use crate::{FlexId, Side, SpatialId, TemporalId, spatial_id::zoom_level::ZoomLevel};

impl FlexId {
    /// 時間窓 `window` に限定して `self - other` を計算する。
    ///
    /// `self` の時間を `window` に切り詰めてから [`difference`](Self::difference) する。
    /// 集合論的に `(A ∩ W) − B = (A − B) ∩ W` と一致する（空間は不変、時間のみ窓で限定）。
    pub fn difference_in_window(
        &self,
        other: &FlexId,
        window: &TemporalId,
    ) -> impl Iterator<Item = FlexId> {
        // self の時間を窓へクリップ。窓と交差しなければ空。
        let clipped_t = match self.temporal().intersection(window) {
            Some(t) => t,
            None => return Vec::new().into_iter(),
        };
        let clipped = FlexId {
            f_zoomlevel: self.f_zoomlevel,
            f_index: self.f_index,
            x_zoomlevel: self.x_zoomlevel,
            x_index: self.x_index,
            y_zoomlevel: self.y_zoomlevel,
            y_index: self.y_index,
            temporal_id: clipped_t,
        };
        clipped.difference(other).collect::<Vec<_>>().into_iter()
    }

    /// 相手の[FlexId]との差集合（self - other）を計算し、イテレータとして返します。
    /// 空間と時間の両方を考慮し、相手にくり抜かれた「残りの領域」を過不足なく細かい FlexId に分割して返します。
    ///
    /// 時間の差分は約数鎖の最小分解で表されるため、時間 `WHOLE` のIDから有限時間の
    /// IDを引いても結果は高々数百セルに収まる（[`TemporalId::difference`] を参照）。
    pub fn difference(&self, other: &FlexId) -> impl Iterator<Item = FlexId> {
        let mut results = Vec::new();

        let intersect = match self.intersection(other) {
            Some(i) => i,
            None => {
                results.push(self.clone());
                return results.into_iter();
            }
        };

        if self == &intersect {
            return results.into_iter();
        }

        let mut current = self.clone();

        while current.f_zoomlevel < intersect.f_zoomlevel {
            let lower = current.split_f(Side::Lower).unwrap();
            let upper = current.split_f(Side::Upper).unwrap();
            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        // X軸の分割
        while current.x_zoomlevel < intersect.x_zoomlevel {
            let lower = current.split_x(Side::Lower).unwrap();
            let upper = current.split_x(Side::Upper).unwrap();

            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        // Y軸の分割
        while current.y_zoomlevel < intersect.y_zoomlevel {
            let lower = current.split_y(Side::Lower).unwrap();
            let upper = current.split_y(Side::Upper).unwrap();

            if lower.intersection(&intersect).is_some() {
                results.push(upper);
                current = lower;
            } else {
                results.push(lower);
                current = upper;
            }
        }

        for t_diff in current.temporal().difference(other.temporal()) {
            results.push(FlexId {
                f_zoomlevel: current.f_zoomlevel,
                f_index: current.f_index,
                x_zoomlevel: current.x_zoomlevel,
                x_index: current.x_index,
                y_zoomlevel: current.y_zoomlevel,
                y_index: current.y_index,
                temporal_id: t_diff,
            });
        }

        results.into_iter()
    }

    /// 2つのFlexIdの重なっている領域（Intersection）を計算して返します。
    /// 重なりがない場合は None を返します。
    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let (f_z, f_i) = Self::intersect_axis_i32(
            self.f_zoomlevel.get(),
            self.f_index,
            other.f_zoomlevel.get(),
            other.f_index,
        )?;

        let (x_z, x_i) = Self::intersect_axis_u32(
            self.x_zoomlevel.get(),
            self.x_index,
            other.x_zoomlevel.get(),
            other.x_index,
        )?;

        let (y_z, y_i) = Self::intersect_axis_u32(
            self.y_zoomlevel.get(),
            self.y_index,
            other.y_zoomlevel.get(),
            other.y_index,
        )?;

        let temporal_id = self.temporal().intersection(other.temporal())?;

        Some(FlexId {
            f_zoomlevel: ZoomLevel::new(f_z).unwrap(),
            f_index: f_i,
            x_zoomlevel: ZoomLevel::new(x_z).unwrap(),
            x_index: x_i,
            y_zoomlevel: ZoomLevel::new(y_z).unwrap(),
            y_index: y_i,
            temporal_id,
        })
    }

    fn intersect_axis_i32(z1: u8, i1: i32, z2: u8, i2: i32) -> Option<(u8, i32)> {
        let (deep_z, deep_i, shallow_z, shallow_i) = if z1 > z2 {
            (z1, i1, z2, i2)
        } else {
            (z2, i2, z1, i1)
        };

        let shift = deep_z - shallow_z;

        if (deep_i >> shift) == shallow_i {
            Some((deep_z, deep_i))
        } else {
            None
        }
    }

    fn intersect_axis_u32(z1: u8, i1: u32, z2: u8, i2: u32) -> Option<(u8, u32)> {
        let (deep_z, deep_i, shallow_z, shallow_i) = if z1 > z2 {
            (z1, i1, z2, i2)
        } else {
            (z2, i2, z1, i1)
        };

        let shift = deep_z - shallow_z;
        if (deep_i >> shift) == shallow_i {
            Some((deep_z, deep_i))
        } else {
            None
        }
    }
}

#[cfg(all(test, feature = "temporal_id"))]
mod temporal_tests {
    use crate::{FlexId, SpatialId, TemporalId};
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    type Atom = ((i32, u32, u32), u64);

    /// FlexId の空間部分を、共通ズーム `z` の単一セル `(f, x, y)` 群へ展開する
    /// （異方セルも各軸を自分のズームから `z` まで展開）。
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

    /// FlexId を (空間キー × 秒) のアトム集合へ展開する。
    fn atoms(f: &FlexId, z: u8) -> BTreeSet<Atom> {
        let secs: Vec<u64> =
            (f.temporal().start_unixtime()..f.temporal().end_unixtime_exclusive()).collect();
        let mut set = BTreeSet::new();
        for k in spatial_keys(f, z) {
            for &s in &secs {
                set.insert((k, s));
            }
        }
        set
    }

    fn atoms_of(fs: &[FlexId], z: u8) -> BTreeSet<Atom> {
        let mut set = BTreeSet::new();
        for f in fs {
            set.extend(atoms(f, z));
        }
        set
    }

    /// 同一空間セル S=(zoom1, f=x=y=0)、時間だけ変える。
    fn st(temp: TemporalId) -> FlexId {
        FlexId::new_with_temporal(1u8, 0, 1u8, 0, 1u8, 0, temp).unwrap()
    }

    /// 同一空間・時間のみ差分：1時間 − 1分 = 59個（空間S × 分セル）。秒断片にならない。
    #[test]
    fn temporal_only_difference() {
        let a = st(TemporalId::from_seconds(3600, 0).unwrap());
        let b = st(TemporalId::from_seconds(60, 0).unwrap());
        let d: Vec<_> = a.difference(&b).collect();
        assert_eq!(d.len(), 59);
        assert!(d.iter().all(|f| f.temporal().i() == 60));
        assert!(
            d.iter()
                .all(|f| f.f_zoomlevel() == 1 && f.x_index() == 0 && f.y_index() == 0)
        );
        let exp: BTreeSet<Atom> = atoms(&a, 1).difference(&atoms(&b, 1)).copied().collect();
        assert_eq!(atoms_of(&d, 1), exp);
    }

    /// WHOLE 時間の FlexId を窓で限定した差分が有界（59分）になる。
    #[test]
    fn windowed_difference_bounds_whole() {
        let a = st(TemporalId::WHOLE);
        let b = st(TemporalId::from_seconds(60, 600).unwrap()); // [36000, 36060)
        let window = TemporalId::from_seconds(3600, 10).unwrap(); // [36000, 39600)
        let d: Vec<_> = a.difference_in_window(&b, &window).collect();
        assert_eq!(d.len(), 59);
        assert!(d.iter().all(|f| f.temporal().i() == 60));
    }

    /// WHOLE 時間の FlexId から有限時間を引いても有界（対数個）のセルで表現される。
    #[test]
    fn whole_difference_is_bounded() {
        let a = st(TemporalId::WHOLE);
        let b = st(TemporalId::from_seconds(60, 600).unwrap()); // [36000, 36060)
        let d: Vec<_> = a.difference(&b).collect();
        assert!(d.len() < 400, "cells = {}", d.len());
        // 空間は不変、時間の被覆は「全時間 − 60秒」
        let total: u64 = d
            .iter()
            .map(|f| f.temporal().end_unixtime_exclusive() - f.temporal().start_unixtime())
            .sum();
        assert_eq!(total, TemporalId::DOMAIN_END - 60);
    }

    /// 空間・時間ともに異なる 4D 差分を (空間キー×秒) のアトムで厳密照合する。
    #[test]
    fn spatio_temporal_difference_atom_oracle() {
        // A = S1(zoom1) × [0,60),  B = S2(zoom2, S2⊂S1) × [0,1)
        let a = FlexId::new_with_temporal(
            1u8,
            0,
            1u8,
            0,
            1u8,
            0,
            TemporalId::from_seconds(60, 0).unwrap(),
        )
        .unwrap();
        let b = FlexId::new_with_temporal(
            2u8,
            0,
            2u8,
            0,
            2u8,
            0,
            TemporalId::from_seconds(1, 0).unwrap(),
        )
        .unwrap();
        let d: Vec<_> = a.difference(&b).collect();
        let got = atoms_of(&d, 2);
        let exp: BTreeSet<Atom> = atoms(&a, 2).difference(&atoms(&b, 2)).copied().collect();
        assert_eq!(got, exp);
        // ピース同士は非交差（アトム総数が一致）
        let total: usize = d.iter().map(|p| atoms(p, 2).len()).sum();
        assert_eq!(total, got.len());
    }

    /// 空間のみ（時間 WHOLE）差分は従来どおり（全ピース時間 WHOLE）。
    #[test]
    fn spatial_only_regression() {
        let a = FlexId::new(1u8, 0, 1u8, 0, 1u8, 0).unwrap();
        let b = FlexId::new(2u8, 0, 2u8, 0, 2u8, 0).unwrap();
        let d: Vec<_> = a.difference(&b).collect();
        assert!(d.iter().all(|f| f.temporal().is_whole()));
        let got: BTreeSet<_> = d.iter().flat_map(|f| spatial_keys(f, 2)).collect();
        let sa: BTreeSet<_> = spatial_keys(&a, 2).into_iter().collect();
        let sb: BTreeSet<_> = spatial_keys(&b, 2).into_iter().collect();
        assert_eq!(got, sa.difference(&sb).copied().collect());
    }
}
