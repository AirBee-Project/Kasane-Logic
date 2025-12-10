// src/id/space_id/single.rs
use itertools::iproduct;
use std::fmt;

use crate::{
    error::Error,
    id::space_id::{
        SpaceID,
        constants::{F_MAX, F_MIN, XY_MAX},
        encode::EncodeID,
        helpers,
        segment::Segment,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct SingleID {
    pub(crate) z: u8,
    pub(crate) f: i64,
    pub(crate) x: u64,
    pub(crate) y: u64,
}

impl fmt::Display for SingleID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}/{}", self.z, self.f, self.x, self.y)
    }
}

impl SingleID {
    /// 指定された値から標準的な空間ID [`SingleID`] を構築します。
    ///
    /// このコンストラクタは、与えられた `z`, `f`, `x`, `y` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返します。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–63の範囲が有効）  
    /// * `f` — Fインデックス（鉛直方向）
    /// * `x` — Xインデックス（東西方向）
    /// * `y` — Yインデックス（南北方向）
    ///
    /// # バリデーション
    /// - `z` が 63 を超える場合、[`Error::ZOutOfRange`] を返します。  
    /// - `f` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、  
    ///   [`Error::FOutOfRange`] を返します。  
    /// - `x` または `y` が `0..=XY_MAX[z]` の範囲外の場合、  
    ///   それぞれ [`Error::XOutOfRange`]、[`Error::YOutOfRange`] を返します。
    ///
    /// # 戻り値
    /// 範囲検証に成功した場合は、対応する [`SingleID`] を返します。
    ///
    /// # 例
    ///
    /// IDの作成
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// let id = SingleID::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.to_string(), "5/3/2/10".to_string());
    /// ```
    ///
    /// 次元の範囲外の検知
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let id = SingleID::new(3, 3, 2, 10);
    /// assert_eq!(id, Err(Error::YOutOfRange{z:3,y:10}));
    /// ```
    ///
    /// ズームレベルの範囲外の検知
    /// ```
    /// # use kasane_logic::id::space_id::single::SingleID;
    /// # use kasane_logic::error::Error;
    /// let id = SingleID::new(68, 3, 2, 10);
    /// assert_eq!(id, Err(Error::ZOutOfRange { z:68 }));
    /// ```

    pub fn new(z: u8, f: i64, x: u64, y: u64) -> Result<SingleID, Error> {
        if z > 63u8 {
            return Err(Error::ZOutOfRange { z });
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f < f_min || f > f_max {
            return Err(Error::FOutOfRange { f, z });
        }
        if x > xy_max {
            return Err(Error::XOutOfRange { x, z });
        }
        if y > xy_max {
            return Err(Error::YOutOfRange { y, z });
        }

        Ok(SingleID { z, f, x, y })
    }

    pub fn as_z(&self) -> &u8 {
        &self.z
    }
    pub fn as_f(&self) -> &i64 {
        &self.f
    }
    pub fn as_x(&self) -> &u64 {
        &self.x
    }
    pub fn as_y(&self) -> &u64 {
        &self.y
    }

    pub fn set_f(&mut self, value: i64) -> Result<(), Error> {
        let min = self.min_f();
        let max = self.max_f();
        if value < min || value > max {
            return Err(Error::FOutOfRange {
                f: value,
                z: self.z,
            });
        }
        self.f = value;
        Ok(())
    }

    pub fn set_x(&mut self, value: u64) -> Result<(), Error> {
        let max = self.max_xy();
        if value > max {
            return Err(Error::XOutOfRange {
                x: value,
                z: self.z,
            });
        }
        self.x = value;
        Ok(())
    }

    pub fn set_y(&mut self, value: u64) -> Result<(), Error> {
        let max = self.max_xy();
        if value > max {
            return Err(Error::YOutOfRange {
                y: value,
                z: self.z,
            });
        }
        self.y = value;
        Ok(())
    }

    pub fn children(&self, difference: u8) -> Result<impl Iterator<Item = SingleID>, Error> {
        let z = self
            .z
            .checked_add(difference)
            .ok_or(Error::ZOutOfRange { z: u8::MAX })?;

        if z > 63 {
            return Err(Error::ZOutOfRange { z });
        }

        let scale_f = 2_i64.pow(difference as u32);
        let scale_xy = 2_u64.pow(difference as u32);

        let f_range = self.f * scale_f..=self.f * scale_f + scale_f - 1;
        let x_range = self.x * scale_xy..=self.x * scale_xy + scale_xy - 1;
        let y_range = self.y * scale_xy..=self.y * scale_xy + scale_xy - 1;

        Ok(iproduct!(f_range, x_range, y_range).map(move |(f, x, y)| SingleID { z, f, x, y }))
    }

    pub fn parent(&self, difference: u8) -> Option<SingleID> {
        let z = self.z.checked_sub(difference)?;
        let f = if self.f == -1 {
            -1
        } else {
            self.f >> difference
        };
        let x = self.x >> (difference as u32);
        let y = self.y >> (difference as u32);
        Some(SingleID { z, f, x, y })
    }
}

impl crate::id::space_id::SpaceID for SingleID {
    fn min_f(&self) -> i64 {
        F_MIN[self.z as usize]
    }
    fn max_f(&self) -> i64 {
        F_MAX[self.z as usize]
    }
    fn max_xy(&self) -> u64 {
        XY_MAX[self.z as usize]
    }

    // bound (error on out-of-range)
    fn bound_up(&mut self, by: i64) -> Result<(), Error> {
        self.f = helpers::checked_add_f_in_range(self.f, by, self.min_f(), self.max_f(), self.z)?;
        Ok(())
    }

    fn bound_down(&mut self, by: i64) -> Result<(), Error> {
        self.f = helpers::checked_sub_f_in_range(self.f, by, self.min_f(), self.max_f(), self.z)?;
        Ok(())
    }

    fn bound_north(&mut self, by: u64) -> Result<(), Error> {
        self.y = helpers::checked_add_u64_in_range(self.y, by, self.max_xy(), |v| {
            Error::YOutOfRange { y: v, z: self.z }
        })?;
        Ok(())
    }

    fn bound_south(&mut self, by: u64) -> Result<(), Error> {
        self.y = helpers::checked_sub_u64_in_range(self.y, by, self.max_xy(), |v| {
            Error::YOutOfRange { y: v, z: self.z }
        })?;
        Ok(())
    }

    fn bound_east(&mut self, by: u64) -> Result<(), Error> {
        self.x = helpers::checked_add_u64_in_range(self.x, by, self.max_xy(), |v| {
            Error::XOutOfRange { x: v, z: self.z }
        })?;
        Ok(())
    }

    fn bound_west(&mut self, by: u64) -> Result<(), Error> {
        self.x = helpers::checked_sub_u64_in_range(self.x, by, self.max_xy(), |v| {
            Error::XOutOfRange { x: v, z: self.z }
        })?;
        Ok(())
    }

    // wrap (cyclic)
    fn wrap_up(&mut self, by: i64) {
        self.f = helpers::wrap_add_i64_in_range(self.f, by, self.min_f(), self.max_f());
    }

    fn wrap_down(&mut self, by: i64) {
        self.f = helpers::wrap_add_i64_in_range(self.f, -by, self.min_f(), self.max_f());
    }

    fn wrap_north(&mut self, by: u64) {
        self.y = helpers::wrap_add_u64(self.y, by, self.max_xy());
    }

    fn wrap_south(&mut self, by: u64) {
        self.y = helpers::wrap_add_u64(
            self.y,
            self.max_xy()
                .wrapping_add(1)
                .wrapping_sub(by % (self.max_xy().wrapping_add(1))),
            self.max_xy(),
        );
    }

    fn wrap_east(&mut self, by: u64) {
        self.x = helpers::wrap_add_u64(self.x, by, self.max_xy());
    }

    fn wrap_west(&mut self, by: u64) {
        self.x = helpers::wrap_add_u64(
            self.x,
            self.max_xy()
                .wrapping_add(1)
                .wrapping_sub(by % (self.max_xy().wrapping_add(1))),
            self.max_xy(),
        );
    }
}

impl From<SingleID> for EncodeID {
    fn from(id: SingleID) -> Self {
        let f_bitvec = Segment { z: id.z, dim: id.f }.to_bitvec();
        let x_bitvec = Segment { z: id.z, dim: id.x }.to_bitvec();
        let y_bitvec = Segment { z: id.z, dim: id.y }.to_bitvec();

        EncodeID {
            f: vec![f_bitvec],
            x: vec![x_bitvec],
            y: vec![y_bitvec],
        }
    }
}
