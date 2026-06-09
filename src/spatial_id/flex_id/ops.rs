#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::{FlexId, Side, SpatialId};

impl FlexId {
    /// 相手の[FlexId]との差集合（self - other）を計算し、イテレータとして返します。
    /// 空間と時間の両方を考慮し、相手にくり抜かれた「残りの領域」を過不足なく細かい FlexId に分割して返します。
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
            self.f_zoomlevel,
            self.f_index,
            other.f_zoomlevel,
            other.f_index,
        )?;

        let (x_z, x_i) = Self::intersect_axis_u32(
            self.x_zoomlevel,
            self.x_index,
            other.x_zoomlevel,
            other.x_index,
        )?;

        let (y_z, y_i) = Self::intersect_axis_u32(
            self.y_zoomlevel,
            self.y_index,
            other.y_zoomlevel,
            other.y_index,
        )?;

        let temporal_id = self.temporal().intersection(other.temporal())?;

        Some(FlexId {
            f_zoomlevel: f_z,
            f_index: f_i,
            x_zoomlevel: x_z,
            x_index: x_i,
            y_zoomlevel: y_z,
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
