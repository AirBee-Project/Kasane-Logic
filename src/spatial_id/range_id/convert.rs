use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{FlexId, RangeId, SingleId, SpatialId, spatial_id::zoom_level::ZoomLevel};

/// XYにおけるセグメントの最適配置関数
pub fn split_xy(z: u8, range: [u32; 2]) -> impl Iterator<Item = (u8, u32)> {
    let [l, r] = range;
    SegmentIter {
        l: l as i64,
        r: r as i64,
        cur_z: z as i8,
    }
    .map(|(z, dim)| (z, dim as u32))
}

/// Fにおけるセグメントの最適配置関数
pub fn split_f(z: u8, range: [i32; 2]) -> impl Iterator<Item = (u8, i32)> {
    let diff = 1i64 << z;
    let [l, r] = range;
    SegmentIter {
        l: l as i64 + diff,
        r: r as i64 + diff,
        cur_z: z as i8,
    }
    .map(move |(seg_z, dim)| {
        let original_dim = dim - (1i64 << seg_z);
        (seg_z, original_dim as i32)
    })
}

impl From<SingleId> for RangeId {
    fn from(id: SingleId) -> Self {
        RangeId::from(&id)
    }
}

impl From<&SingleId> for RangeId {
    fn from(id: &SingleId) -> Self {
        RangeId {
            z: ZoomLevel::new(id.z()).unwrap(),
            f: [id.f(), id.f()],
            x: [id.x(), id.x()],
            y: [id.y(), id.y()],

            temporal: id.temporal(),
        }
    }
}

impl RangeId {
    /// この [`RangeId`] が覆う空間セルを1つずつ [`SingleId`] として列挙する。
    ///
    /// 時間軸は分解しない。[`SingleId`]も[`RangeId`]と同じ[`TemporalId`](crate::TemporalId)を
    /// 保持できるため、各 [`SingleId`] は元の時間区間をそのまま引き継ぐ。
    pub fn single_ids(self) -> Box<dyn Iterator<Item = SingleId>> {
        let z = self.z.get();
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];
        let temporal = self.temporal.clone();

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let temporal = temporal.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=ZoomLevel::new(z).unwrap().xy_max())
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                let temporal = temporal.clone();
                y_range.clone().map(move |y: u32| {
                    SingleId::new(z, f, x, y)
                        .unwrap()
                        .with_temporal(temporal.clone())
                })
            })
        });
        Box::new(iter)
    }
}

impl IntoIterator for RangeId {
    type Item = FlexId;
    type IntoIter = Box<dyn Iterator<Item = FlexId>>;

    fn into_iter(self) -> Self::IntoIter {
        let z = self.z.get();
        #[allow(clippy::needless_collect)]
        let f_list: Vec<_> = split_f(z, self.f).collect();

        let x_list: Vec<_> = if self.x[0] <= self.x[1] {
            split_xy(z, self.x).collect()
        } else {
            split_xy(z, [self.x[0], ZoomLevel::new(z).unwrap().xy_max()])
                .chain(split_xy(z, [0, self.x[1]]))
                .collect()
        };
        let y_list: Vec<_> = split_xy(z, self.y).collect();

        #[cfg(feature = "temporal_id")]
        {
            let t_list: Vec<(u8, u64)> = self
                .temporal
                .segments()
                .map(|c| (c.zoom(), c.index()))
                .collect();

            let iter = f_list.into_iter().flat_map(move |(f_z, f_i)| {
                let y_list_inner = y_list.clone();
                let t_list_inner = t_list.clone();
                x_list.clone().into_iter().flat_map(move |(x_z, x_i)| {
                    let y_list_inner = y_list_inner.clone();
                    let t_list_inner = t_list_inner.clone();
                    y_list_inner
                        .clone()
                        .into_iter()
                        .flat_map(move |(y_z, y_i)| {
                            t_list_inner.clone().into_iter().map(move |(t_z, t_i)| {
                                unsafe { FlexId::new_unchecked(f_z, f_i, x_z, x_i, y_z, y_i) }
                                    .with_raw_temporal(unsafe {
                                        crate::TemporalSegment::new_unchecked(t_z, t_i)
                                    })
                            })
                        })
                })
            });
            Box::new(iter)
        }

        #[cfg(not(feature = "temporal_id"))]
        {
            let iter = f_list.into_iter().flat_map(move |(f_z, f_i)| {
                let y_list_inner = y_list.clone();
                x_list.clone().into_iter().flat_map(move |(x_z, x_i)| {
                    let y_list_inner = y_list_inner.clone();
                    y_list_inner.into_iter().map(move |(y_z, y_i)| unsafe {
                        FlexId::new_unchecked(f_z, f_i, x_z, x_i, y_z, y_i)
                    })
                })
            });
            Box::new(iter)
        }
    }
}

/// 区間木的にセグメントを最適配置するイテレータ。
///
/// `l`/`r`は`i32`ではなく`i64`で保持する。[`split_f`]がFのインデックスへ`2^z`のオフセットを
/// 加えるため、`z = ZoomLevel::MAX`（30）かつFが最大値のときに`2^31 - 1`（`i32::MAX`）へ達し、
/// 最後の要素を返した直後の`l += 1`が`i32`ではオーバーフローするからである。
struct SegmentIter {
    l: i64,
    r: i64,
    cur_z: i8,
}

impl Iterator for SegmentIter {
    type Item = (u8, i64); // (z, dimension)

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.l > self.r {
                return None;
            }

            if self.cur_z == 0 {
                let v = self.l;
                self.l += 1;
                return Some((0, v));
            }

            let z = self.cur_z as u8;
            if self.l == self.r {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.l & 1 == 1 {
                let v = self.l;
                self.l += 1;
                return Some((z, v));
            }
            if self.r & 1 == 0 {
                let v = self.r;
                self.r -= 1;
                return Some((z, v));
            }
            self.l >>= 1;
            self.r >>= 1;
            self.cur_z -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ZoomLevel;
    use alloc::vec::Vec;

    /// `split_f`はFのインデックスへ`2^z`を足すため、最深ズームかつF最大値では
    /// `i32::MAX`ちょうどに達する。内部を`i64`で保持していないと、最後の要素を返した直後の
    /// `l += 1`がオーバーフローしてパニックする。
    #[test]
    fn split_f_does_not_overflow_at_deepest_zoom_upper_bound() {
        let z = ZoomLevel::MAX.get();
        let f_max = ZoomLevel::MAX.f_max();

        let got: Vec<_> = split_f(z, [f_max, f_max]).collect();
        assert_eq!(got, [(z, f_max)]);

        let f_min = ZoomLevel::MAX.f_min();
        let got: Vec<_> = split_f(z, [f_min, f_min]).collect();
        assert_eq!(got, [(z, f_min)]);
    }

    /// 同じ境界を [`RangeId`] の展開経路からも踏む（[`SingleId`]は内部でこれを通る）。
    #[test]
    fn range_id_at_deepest_zoom_upper_bound_expands() {
        let z = ZoomLevel::MAX.get();
        let f_max = ZoomLevel::MAX.f_max();
        let xy_max = ZoomLevel::MAX.xy_max();

        let id = SingleId::new(z, f_max, xy_max, xy_max).unwrap();
        assert_eq!(id.into_iter().count(), 1);
    }
}
