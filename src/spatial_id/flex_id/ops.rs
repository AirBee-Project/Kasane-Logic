#[cfg(feature = "temporal_id")]
use crate::SpatialId;
#[cfg(feature = "temporal_id")]
use crate::spatial_id::zoom_level::TZoomLevel;
use crate::{FlexId, Side, spatial_id::zoom_level::ZoomLevel};
use alloc::vec::Vec;

impl FlexId {
    /// 相手の [`FlexId`] との差集合（`self - other`）を計算し、イテレータとして返します。
    ///
    /// 4軸（F / X / Y / T）とも同じ手順で、「交差と同じ深さになるまで2分し、交差しない側を
    /// 結果へ確定させる」ことを繰り返します。
    pub fn difference(&self, other: &FlexId) -> impl Iterator<Item = FlexId> + use<> {
        let mut results = Vec::new();

        let intersect = match self.intersection(other) {
            Some(i) => i,
            None => {
                results.push(*self);
                return results.into_iter();
            }
        };

        if self == &intersect {
            return results.into_iter();
        }

        let mut current = *self;

        // 軸ごとに違うのは「その軸のズームを取る方法」と「その軸で2分する方法」だけ。
        // `temporal_id` 無効時はTの軸自体が無いので、空回りする1周ごと削る。
        //
        // `node::Axis`（FlexTreeが分割する軸そのもの）とは別物なので、同じ名前を避けて
        // `AxisOps` にしている（これは軸の値ではなく、軸ごとの操作関数のペア）。
        type AxisOps = (fn(&FlexId) -> u8, fn(&FlexId, Side) -> Option<FlexId>);
        #[cfg(feature = "temporal_id")]
        const AXES: [AxisOps; 4] = [
            (FlexId::f_zoomlevel, FlexId::split_f),
            (FlexId::x_zoomlevel, FlexId::split_x),
            (FlexId::y_zoomlevel, FlexId::split_y),
            (FlexId::t_zoomlevel, FlexId::split_t),
        ];
        #[cfg(not(feature = "temporal_id"))]
        const AXES: [AxisOps; 3] = [
            (FlexId::f_zoomlevel, FlexId::split_f),
            (FlexId::x_zoomlevel, FlexId::split_x),
            (FlexId::y_zoomlevel, FlexId::split_y),
        ];

        for (zoom_of, split) in AXES {
            while zoom_of(&current) < zoom_of(&intersect) {
                let lower = split(&current, Side::Lower).expect("交差より浅いので必ず割れる");
                let upper = split(&current, Side::Upper).expect("交差より浅いので必ず割れる");
                if lower.intersection(&intersect).is_some() {
                    results.push(upper);
                    current = lower;
                } else {
                    results.push(lower);
                    current = upper;
                }
            }
        }

        results.into_iter()
    }

    /// 2つの [`FlexId`] の重なっている領域（Intersection）を計算して返します。
    /// 重なりがない場合は [`None`] を返します。
    ///
    /// どの軸も「浅い側のSegmentが深い側を含むか」で判定する。四分木（八分木）／2進トライでは
    /// 異なるズームのSegmentは入れ子か素のどちらかしかないので、重なる場合の交差は必ず
    /// 深い側のSegmentそのものになる。
    pub fn intersection(&self, other: &FlexId) -> Option<FlexId> {
        let (f_z, f_i) = nested_axis(
            self.f_zoomlevel(),
            self.f_index() as i64,
            other.f_zoomlevel(),
            other.f_index() as i64,
        )?;
        let (x_z, x_i) = nested_axis(
            self.x_zoomlevel(),
            self.x_index() as i64,
            other.x_zoomlevel(),
            other.x_index() as i64,
        )?;
        let (y_z, y_i) = nested_axis(
            self.y_zoomlevel(),
            self.y_index() as i64,
            other.y_zoomlevel(),
            other.y_index() as i64,
        )?;

        // 時間軸は `temporal_id` 無効時には存在しない（双方とも全時間で必ず入れ子）ので、
        // 判定ごと消して空間3軸だけの費用に戻す。
        #[cfg(feature = "temporal_id")]
        let (t_z, t_i) = nested_axis(
            self.t_zoomlevel(),
            self.t() as i64,
            other.t_zoomlevel(),
            other.t() as i64,
        )?;

        Some(FlexId {
            f_zoomlevel: ZoomLevel::new(f_z).unwrap(),
            f_index: f_i as i32,
            x_zoomlevel: ZoomLevel::new(x_z).unwrap(),
            x_index: x_i as u32,
            y_zoomlevel: ZoomLevel::new(y_z).unwrap(),
            y_index: y_i as u32,
            #[cfg(feature = "temporal_id")]
            t_zoomlevel: TZoomLevel::new(t_z).unwrap(),
            #[cfg(feature = "temporal_id")]
            t_index: t_i as u64,
        })
    }

    /// [`RangeId`](crate::RangeId) と交差するか判定する。**時間軸も含めて**判定する。
    ///
    /// 木の走査（`RangeOverlapWalk`）は枝刈りで大半を落とすが、時間軸は
    /// Segmentの2分割境界とターゲットの秒区間が一致するとは限らないため、はみ出した葉が
    /// 残りうる。ここが最終フィルタである。
    pub fn intersects_range(&self, range: &crate::RangeId) -> bool {
        // 時間軸だけは「共通ズームでの整数範囲」に落とせない（`RangeId` の `Interval` は
        // 2の冪とは限らない）ので、絶対秒区間の重なりで判定する。
        // `temporal_id` 無効時は双方とも全時間で必ず重なるため、判定ごと消す。
        #[cfg(feature = "temporal_id")]
        {
            let (self_start, self_end) = self.seconds_range();
            let (range_start, range_end) = range.seconds_range();
            if self_start >= range_end || range_start >= self_end {
                return false;
            }
        }

        overlaps_axis(
            self.f_zoomlevel(),
            self.f_index() as i64,
            range.z(),
            range.f()[0] as i64,
            range.f()[1] as i64,
        ) && overlaps_axis(
            self.x_zoomlevel(),
            self.x_index() as i64,
            range.z(),
            range.x()[0] as i64,
            range.x()[1] as i64,
        ) && overlaps_axis(
            self.y_zoomlevel(),
            self.y_index() as i64,
            range.z(),
            range.y()[0] as i64,
            range.y()[1] as i64,
        )
    }
}

/// 1軸について、2つのSegmentが入れ子なら「深い側」の `(zoom, index)` を返す。素なら [`None`]。
///
/// F は符号付き `i32`、X/Y は `u32`、T は `u64` と幅が違うが判定式は同じなので、`i64` へ
/// 揃えて1つの関数で扱う（このクレートが扱う範囲——`u32` の全域と `2^62` までの `u64`——は
/// `i64` で情報を落とさずに表せる）。
fn nested_axis(z1: u8, i1: i64, z2: u8, i2: i64) -> Option<(u8, i64)> {
    let (deep_z, deep_i, shallow_z, shallow_i) = if z1 > z2 {
        (z1, i1, z2, i2)
    } else {
        (z2, i2, z1, i1)
    };

    let shift = deep_z - shallow_z;
    ((deep_i >> shift) == shallow_i).then_some((deep_z, deep_i))
}

/// 1軸について、Segmentと（別ズームの）整数範囲が重なるか。
fn overlaps_axis(
    segment_z: u8,
    segment_i: i64,
    range_z: u8,
    range_min: i64,
    range_max: i64,
) -> bool {
    let (deep_z, deep_min, deep_max, shallow_z, shallow_min, shallow_max) = if segment_z > range_z {
        (
            segment_z, segment_i, segment_i, range_z, range_min, range_max,
        )
    } else {
        (
            range_z, range_min, range_max, segment_z, segment_i, segment_i,
        )
    };
    let shift = deep_z - shallow_z;
    !((deep_max >> shift) < shallow_min || (deep_min >> shift) > shallow_max)
}
