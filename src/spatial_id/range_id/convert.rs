use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{FlexId, RangeId, SingleId, SpatialId, spatial_id::zoom_level::ZoomLevel};

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

            temporal_id: id.temporal().clone(),
        }
    }
}

impl RangeId {
    pub fn single_ids(self) -> Box<dyn Iterator<Item = SingleId>> {
        let z = self.z.get();
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];
        let t_id = self.temporal_id.clone();

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let t_id = t_id.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=ZoomLevel::new(z).unwrap().xy_max())
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                let t_id = t_id.clone();
                y_range.clone().map(move |y: u32| {
                    #[cfg(feature = "temporal_id")]
                    {
                        SingleId::new_with_temporal(z, f, x, y, t_id.clone()).unwrap()
                    }

                    #[cfg(not(feature = "temporal_id"))]
                    {
                        let _ = &t_id;
                        SingleId::new(z, f, x, y).unwrap()
                    }
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

        let iter = f_list.into_iter().flat_map(move |(f_z, f_i)| {
            let y_list_inner = y_list.clone();
            x_list.clone().into_iter().flat_map(move |(x_z, x_i)| {
                let y_list_inner = y_list_inner.clone();
                y_list_inner
                    .into_iter()
                    .map(move |(y_z, y_i)| FlexId::new(f_z, f_i, x_z, x_i, y_z, y_i).unwrap())
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
