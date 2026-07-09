use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{
    FlexId, IterFlexIds, IterSingleIds, RangeId, SingleId, SpatialId,
    spatial_id::zoom_level::ZoomLevel,
};

/// XYにおけるセグメントの最適配置関数
pub fn split_xy(z: u8, range: [u32; 2]) -> impl Iterator<Item = (u8, u32)> {
    let [l, r] = range;
    SegmentIter {
        l: l as i32,
        r: r as i32,
        cur_z: z as i8,
    }
    .map(|(z, dim)| (z, dim as u32))
}

/// Fにおけるセグメントの最適配置関数
pub fn split_f(z: u8, range: [i32; 2]) -> impl Iterator<Item = (u8, i32)> {
    let diff = (1i64 << z) as i32;
    let [l, r] = range;
    SegmentIter {
        l: l + diff,
        r: r + diff,
        cur_z: z as i8,
    }
    .map(move |(seg_z, dim)| {
        let original_dim = dim - ((1i64 << seg_z) as i32);
        (seg_z, original_dim)
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

            temporal_id: id.temporal(),
        }
    }
}

impl RangeId {
    /// この [`RangeId`] を消費し、含まれる [`SingleId`] を所有イテレータとして返す。
    ///
    /// [`iter_single_ids`](IterSingleIds::iter_single_ids) の所有版。`self` を借用しないため、
    /// 一時的な `RangeId` から `flat_map` などで直接ストリーミングでき、中間 `Vec` が不要になる。
    pub fn single_ids(self) -> impl Iterator<Item = SingleId> {
        let z = self.z.get();
        let (f0, f1) = (self.f[0], self.f[1]);
        let (x0, x1) = (self.x[0], self.x[1]);
        let (y0, y1) = (self.y[0], self.y[1]);
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal_id;

        (f0..=f1).flat_map(move |f| {
            // X は経度方向に巡回するため、境界跨ぎ（x0 > x1）は2区間に分ける。
            let x_iter: Vec<u32> = if x0 <= x1 {
                (x0..=x1).collect()
            } else {
                (x0..=ZoomLevel::new(z).unwrap().xy_max())
                    .chain(0..=x1)
                    .collect()
            };
            #[cfg(feature = "temporal_id")]
            let temporal_id = temporal_id;

            x_iter.into_iter().flat_map(move |x| {
                #[cfg(feature = "temporal_id")]
                let temporal_id = temporal_id;
                (y0..=y1).map(move |y: u32| {
                    #[cfg(feature = "temporal_id")]
                    {
                        SingleId::new(z, f, x, y)
                            .unwrap()
                            .with_temporal(temporal_id)
                    }

                    #[cfg(not(feature = "temporal_id"))]
                    {
                        SingleId::new(z, f, x, y).unwrap()
                    }
                })
            })
        })
    }
}

impl IterSingleIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = SingleId> + 'a>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        Box::new(self.clone().single_ids())
    }
}

impl IterFlexIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = FlexId> + 'a>;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        let z = self.z.get();
        let f_list: Vec<_> = split_f(z, self.f).collect();

        let x_list: Vec<_> = if self.x[0] <= self.x[1] {
            split_xy(z, self.x).collect()
        } else {
            split_xy(z, [self.x[0], ZoomLevel::new(z).unwrap().xy_max()])
                .chain(split_xy(z, [0, self.x[1]]))
                .collect()
        };
        let y_list: Vec<_> = split_xy(z, self.y).collect();

        let t_id = self.temporal_id;
        let iter = f_list.into_iter().flat_map(move |(f_z, f_i)| {
            let y_list_inner = y_list.clone();
            let x_list_inner = x_list.clone();
            let t_id_inner = t_id;
            x_list_inner.into_iter().flat_map(move |(x_z, x_i)| {
                let y_list_inner2 = y_list_inner.clone();
                let _t_id_inner2 = t_id_inner;
                y_list_inner2.into_iter().map(move |(y_z, y_i)| {
                    #[cfg(feature = "temporal_id")]
                    {
                        FlexId::new(f_z, f_i, x_z, x_i, y_z, y_i)
                            .unwrap()
                            .with_temporal(_t_id_inner2)
                    }
                    #[cfg(not(feature = "temporal_id"))]
                    {
                        FlexId::new(f_z, f_i, x_z, x_i, y_z, y_i).unwrap()
                    }
                })
            })
        });
        Box::new(iter)
    }
}

struct SegmentIter {
    l: i32,
    r: i32,
    cur_z: i8,
}

impl Iterator for SegmentIter {
    type Item = (u8, i32); // (z, dimension)

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
