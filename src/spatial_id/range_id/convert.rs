use crate::{
    FlexId, IntoFlexIds, IntoSingleIds, IterFlexIds, IterSingleIds, RangeId, SingleId, SpatialId,
    XY_MAX,
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
    let diff = 1i32 << z;
    let [l, r] = range;
    SegmentIter {
        l: l + diff,
        r: r + diff,
        cur_z: z as i8,
    }
    .map(move |(seg_z, dim)| {
        let original_dim = dim - (1i32 << seg_z);
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
            z: id.z(),
            f: [id.f(), id.f()],
            x: [id.x(), id.x()],
            y: [id.y(), id.y()],

            temporal_id: id.temporal().clone(),
        }
    }
}

impl IntoSingleIds for RangeId {
    type IntoIter = Box<dyn Iterator<Item = SingleId>>;
    fn into_single_ids(self) -> Self::IntoIter {
        let z = self.z;
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];
        let t_id = self.temporal_id.clone();

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();
            let t_id = t_id.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=XY_MAX[z as usize])
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                let t_id = t_id.clone();
                y_range.clone().map(move |y: u32| {
                    #[cfg(feature = "temporal_id")]
                    {
                        unsafe { SingleId::new_with_temporal_unchecked(z, f, x, y, t_id.clone()) }
                    }

                    #[cfg(not(feature = "temporal_id"))]
                    {
                        let _ = &t_id;
                        unsafe { SingleId::new_unchecked(z, f, x, y) }
                    }
                })
            })
        });
        Box::new(iter)
    }
}

impl IterSingleIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = SingleId> + 'a>;
    fn iter_single_ids(&self) -> Self::Iter<'_> {
        let z = self.z;
        let f_range = self.f[0]..=self.f[1];
        let y_range = self.y[0]..=self.y[1];

        let iter = f_range.flat_map(move |f| {
            let y_range = y_range.clone();

            let x_iter = if self.x[0] <= self.x[1] {
                (self.x[0]..=self.x[1]).collect::<Vec<_>>()
            } else {
                (self.x[0]..=XY_MAX[z as usize])
                    .chain(0..=self.x[1])
                    .collect::<Vec<_>>()
            };

            x_iter.into_iter().flat_map(move |x| {
                y_range.clone().map(move |y: u32| {
                    #[cfg(feature = "temporal_id")]
                    {
                        unsafe {
                            SingleId::new_with_temporal_unchecked(
                                z,
                                f,
                                x,
                                y,
                                self.temporal_id.clone(),
                            )
                        }
                    }

                    #[cfg(not(feature = "temporal_id"))]
                    {
                        unsafe { SingleId::new_unchecked(z, f, x, y) }
                    }
                })
            })
        });
        Box::new(iter)
    }
}

impl IntoFlexIds for RangeId {
    type IntoIter = Box<dyn Iterator<Item = FlexId>>;

    fn into_flex_ids(self) -> Self::IntoIter {
        let f_list: Vec<_> = split_f(self.z, self.f).collect();

        let x_list: Vec<_> = if self.x[0] <= self.x[1] {
            split_xy(self.z, self.x).collect()
        } else {
            split_xy(self.z, [self.x[0], XY_MAX[self.z as usize]])
                .chain(split_xy(self.z, [0, self.x[1]]))
                .collect()
        };
        let y_list: Vec<_> = split_xy(self.z, self.y).collect();

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

impl IterFlexIds for RangeId {
    type Iter<'a> = Box<dyn Iterator<Item = FlexId> + 'a>;

    fn iter_flex_ids(&self) -> Self::Iter<'_> {
        self.clone().into_flex_ids()
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
