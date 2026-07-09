use alloc::boxed::Box;

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
    pub fn single_ids(self) -> impl Iterator<Item = SingleId> {
        let z = self.z.get();
        let (f0, f1) = (self.f[0], self.f[1]);
        let (x0, x1) = (self.x[0], self.x[1]);
        let (y0, y1) = (self.y[0], self.y[1]);
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal_id;

        (f0..=f1).flat_map(move |f| {
            // X は経度方向に巡回するため、境界跨ぎ（x0 > x1）は2区間に分ける。
            // Vec を経由せず、2分岐を Box<dyn Iterator> で統一して直接ストリームする。
            let xy_max = ZoomLevel::new(z).unwrap().xy_max();
            let x_iter: Box<dyn Iterator<Item = u32>> = if x0 <= x1 {
                Box::new(x0..=x1)
            } else {
                Box::new((x0..=xy_max).chain(0..=x1))
            };
            #[cfg(feature = "temporal_id")]
            let temporal_id = temporal_id;

            x_iter.flat_map(move |x| {
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
        // f / x / y の範囲はすべてコピー可能な値なので、各 flat_map 内で
        // split_* を再計算することで Vec 確保とクローンを避ける。
        let f_range = self.f;
        let x_range = self.x;
        let y_range = self.y;
        let x_wraps = x_range[0] > x_range[1];
        let xy_max = ZoomLevel::new(z).unwrap().xy_max();

        let t_id = self.temporal_id;
        let iter = split_f(z, f_range).flat_map(move |(f_z, f_i)| {
            // X は巡回があるため 2 区間を chain してそれぞれ再計算する。
            let x_iter: Box<dyn Iterator<Item = (u8, u32)>> = if x_wraps {
                Box::new(split_xy(z, [x_range[0], xy_max]).chain(split_xy(z, [0, x_range[1]])))
            } else {
                Box::new(split_xy(z, x_range))
            };
            x_iter.flat_map(move |(x_z, x_i)| {
                split_xy(z, y_range).map(move |(y_z, y_i)| {
                    #[cfg(feature = "temporal_id")]
                    {
                        FlexId::new(f_z, f_i, x_z, x_i, y_z, y_i)
                            .unwrap()
                            .with_temporal(t_id)
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
