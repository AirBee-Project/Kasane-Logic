use std::fmt;

use crate::{
    Coordinate, Ecef, Error, F_MAX, F_MIN, FlexId, SpatialId, SpatialIdError, TemporalId, XY_MAX,
    spatial_id::helpers,
};

impl fmt::Display for FlexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //空間の情報の書き込み
        write!(
            f,
            "{}/{}|{}/{}|{}/{}",
            self.f_zoomlevel,
            self.f_index,
            self.x_zoomlevel,
            self.x_index,
            self.y_zoomlevel,
            self.y_index
        )?;

        //時間の情報があれば書き込み
        if !self.temporal_id.is_whole() {
            write!(f, "_{}", self.temporal_id)?;
        };
        Ok(())
    }
}

impl SpatialId for FlexId {
    fn f_min(&self) -> i32 {
        F_MIN[self.f_zoomlevel as usize]
    }

    fn f_max(&self) -> i32 {
        F_MAX[self.f_zoomlevel as usize]
    }

    fn x_max(&self) -> u32 {
        XY_MAX[self.x_zoomlevel as usize]
    }

    fn y_max(&self) -> u32 {
        XY_MAX[self.y_zoomlevel as usize]
    }

    fn move_f(&mut self, by: i32) -> Result<(), crate::Error> {
        let new = self.f_index.checked_add(by).ok_or_else(|| {
            Error::from(SpatialIdError::FOutOfRange {
                f: if by >= 0 { i32::MAX } else { i32::MIN },
                z: self.f_zoomlevel,
            })
        })?;

        if new < self.f_min() || new > self.f_max() {
            return Err(SpatialIdError::FOutOfRange {
                f: new,
                z: self.f_zoomlevel,
            }
            .into());
        }

        self.f_index = new;
        Ok(())
    }

    fn move_x(&mut self, by: i32) {
        let max_len = (self.x_max() + 1) as i32;
        let new = (self.x_index as i32 + by).rem_euclid(max_len);
        self.x_index = new as u32;
    }

    fn move_y(&mut self, by: i32) -> Result<(), crate::Error> {
        let new = if by >= 0 {
            self.y_index.checked_add(by as u32).ok_or_else(|| {
                Error::from(SpatialIdError::YOutOfRange {
                    y: u32::MAX,
                    z: self.y_zoomlevel,
                })
            })?
        } else {
            self.y_index
                .checked_sub(-by as u32)
                .ok_or(SpatialIdError::YOutOfRange {
                    y: self.y_min(),
                    z: self.y_zoomlevel,
                })?
        };

        if new > self.y_max() {
            return Err(SpatialIdError::YOutOfRange {
                y: new,
                z: self.y_zoomlevel,
            }
            .into());
        }

        self.y_index = new;

        Ok(())
    }

    fn length_f_meters(&self) -> f64 {
        2_f64.powi(25 - self.f_zoomlevel() as i32)
    }

    fn length_x_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(self.x_zoomlevel() as u32) as f64)
    }

    fn length_y_meters(&self) -> f64 {
        let ecef: Ecef = self.spatial_center().into();
        let r = (ecef.x() * ecef.x() + ecef.y() * ecef.y()).sqrt();
        r * 2.0 * std::f64::consts::PI / (2_i32.pow(self.y_zoomlevel() as u32) as f64)
    }

    fn spatial_center(&self) -> crate::Coordinate {
        unsafe {
            Coordinate::new_unchecked(
                helpers::latitude(self.y_index as f64 + 0.5, self.y_zoomlevel),
                helpers::longitude(self.x_index as f64 + 0.5, self.x_zoomlevel),
                helpers::altitude(self.f_index as f64 + 0.5, self.f_zoomlevel),
            )
        }
    }

    fn spatial_vertices(&self) -> [crate::Coordinate; 8] {
        let xs = [self.x_index as f64, self.x_index as f64 + 1.0];
        let ys = [self.y_index as f64, self.y_index as f64 + 1.0];
        let fs = [self.f_index as f64, self.f_index as f64 + 1.0];

        // 各端点の値を前計算しておく
        let lon2 = [
            helpers::longitude(xs[0], self.x_zoomlevel),
            helpers::longitude(xs[1], self.x_zoomlevel),
        ];
        let lat2 = [
            helpers::latitude(ys[0], self.y_zoomlevel),
            helpers::latitude(ys[1], self.y_zoomlevel),
        ];
        let alt2 = [
            helpers::altitude(fs[0], self.f_zoomlevel),
            helpers::altitude(fs[1], self.f_zoomlevel),
        ];

        // 結果配列
        let mut out = [Coordinate::default(); 8];

        let mut i = 0;
        for f_i in 0..2 {
            for y_i in 0..2 {
                for x_i in 0..2 {
                    out[i]
                        .set_longitude(lon2[x_i])
                        .expect("longitude must be within valid range");
                    out[i]
                        .set_latitude(lat2[y_i])
                        .expect("latitude must be within valid range");
                    out[i]
                        .set_altitude(alt2[f_i])
                        .expect("altitude must be within valid range");
                    i += 1;
                }
            }
        }

        out
    }

    fn temporal(&self) -> &TemporalId {
        &self.temporal_id
    }

    fn temporal_mut(&mut self) -> &mut TemporalId {
        &mut self.temporal_id
    }
}
