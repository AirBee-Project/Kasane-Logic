pub mod constructor;
pub mod convert;
pub mod impls;
pub mod random;

use crate::{
    SpatialId, SpatialIdError, TemporalId,
    error::Error,
    spatial_id::{helpers, zoom_level::ZoomLevel},
};

/// RangeIdは空間IDの範囲表現を表す型です。
///
/// 各インデックスを範囲で指定することができます。各次元の範囲を表す配列の順序には意味を持ちません。内部的には下記のような構造体で構成されており、各フィールドをプライベートにすることで、ズームレベルに依存するインデックス範囲やその他のバリデーションを適切に適用することができます。
///
/// この型は `PartialOrd` / `Ord` を実装していますが、これは主に`BTreeSet` や `BTreeMap` などの順序付きコレクションでの格納・探索用です。実際の空間的な「大小」を意味するものではありません。
///
/// ```
/// # use kasane_logic::ZoomLevel;
/// pub struct RangeId {
///     z: ZoomLevel,
///     f: [i32; 2],
///     x: [u32; 2],
///     y: [u32; 2],
/// }
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RangeId {
    z: ZoomLevel,
    f: [i32; 2],
    x: [u32; 2],
    y: [u32; 2],
    temporal_id: TemporalId,
}

impl RangeId {
    /// この `RangeId` が保持しているズームレベル `z` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.z(), 5u8);
    /// ```
    pub fn z(&self) -> u8 {
        self.z.get()
    }

    /// この `RangeId` が保持しているズームレベル `[f1,f2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.f(), [-3i32,29i32]);
    /// ```
    pub fn f(&self) -> [i32; 2] {
        self.f
    }

    /// この `RangeId` が保持しているズームレベル `[x1,x2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.x(), [8u32,9u32]);
    /// ```
    pub fn x(&self) -> [u32; 2] {
        self.x
    }

    /// この `RangeId` が保持しているズームレベル `[y1,y2]` を返します。
    ///
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// assert_eq!(id.y(), [5u32,10u32]);
    /// ```
    pub fn y(&self) -> [u32; 2] {
        self.y
    }

    pub fn set_f(&mut self, value: [i32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let f_min = unsafe { ZoomLevel::new_unchecked(z) }.f_min();
        let f_max = unsafe { ZoomLevel::new_unchecked(z) }.f_max();

        for &f_value in &value {
            if f_value < f_min || f_value > f_max {
                return Err(SpatialIdError::FOutOfRange { f: f_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.f = value;
        Ok(())
    }

    pub fn set_x(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let xy_max = unsafe { ZoomLevel::new_unchecked(z) }.xy_max();

        for &x_value in &value {
            if x_value > xy_max {
                return Err(SpatialIdError::XOutOfRange { x: x_value, z }.into());
            }
        }

        self.x = value;
        Ok(())
    }

    pub fn set_y(&mut self, value: [u32; 2]) -> Result<(), Error> {
        let z = self.z.get();
        let mut value = value;
        let xy_max = unsafe { ZoomLevel::new_unchecked(z) }.xy_max();

        for &y_value in &value {
            if y_value > xy_max {
                return Err(SpatialIdError::YOutOfRange { y: y_value, z }.into());
            }
        }

        if value[0] > value[1] {
            value.swap(0, 1);
        }

        self.y = value;
        Ok(())
    }

    /// 指定したズームレベル `target_z` に細分化した、この `RangeId` を含むすべての子 `RangeId` を生成します。
    ///
    /// # パラメータ
    /// * `target_z` — 生成したい子 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより浅い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段深いズームへの細分化
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(6).unwrap();
    /// assert_eq!(result,  RangeId::new(6, [-6, 59], [16, 19], [10, 21] ).unwrap());
    ///
    /// ```
    ///
    /// 現在より浅いズームを指定した場合
    /// ```
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-3,29], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_children_at_zoom(4);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 4 }))));
    /// ```
    pub fn spatial_children_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z < z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let difference = target_z - z;
        let scale_f = 2_i32.pow(difference as u32);
        let scale_xy = 2_u32.pow(difference as u32);

        let f = helpers::scale_range_i32(self.f[0], self.f[1], scale_f);
        let x = helpers::scale_range_u32(self.x[0], self.x[1], scale_xy);
        let y = helpers::scale_range_u32(self.y[0], self.y[1], scale_xy);

        Ok(RangeId {
            z: unsafe { ZoomLevel::new_unchecked(target_z) },
            f,
            x,
            y,

            temporal_id: self.temporal().clone(),
        })
    }

    /// 指定したズームレベル `target_z` に縮約した、この `RangeId` の親 `RangeId` を返します。
    ///
    /// # パラメータ
    /// * `target_z` — 取得したい親 `RangeId` のズームレベル
    ///
    /// # バリデーション
    /// - `target_z` が現在のズームレベルより深い場合は、[`SpatialIdError::ZoomLevelTransitionOutOfRange`] を返します。
    /// - `target_z` が本クレートで扱える最大ズームレベルを超える場合は、[`SpatialIdError::ZOutOfRange`] を返します。
    ///
    /// 1段浅いズームへの縮約
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [1,29], [8,9], [5,10]).unwrap();
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [0,14]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// Fが負の場合の挙動:
    /// ```
    /// # use kasane_logic::RangeId;
    /// # use kasane_logic::Error;
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    ///
    /// let parent = id.spatial_parent_at_zoom(4).unwrap();
    ///
    /// assert_eq!(parent.z(), 4);
    /// assert_eq!(parent.f(), [-5,-3]);
    /// assert_eq!(parent.x(), [4,4]);
    /// assert_eq!(parent.y(), [2,5]);
    /// ```
    ///
    /// 現在より深いズームを指定した場合:
    /// ```
    /// # use kasane_logic::{Error, RangeId, SpatialIdError};
    /// let id = RangeId::new(5, [-10,-5], [8,9], [5,10]).unwrap();
    /// let result = id.spatial_parent_at_zoom(6);
    /// assert!(matches!(result, Err(Error::SpatialId(SpatialIdError::ZoomLevelTransitionOutOfRange { current_z: 5, target_z: 6 }))));
    /// ```
    pub fn spatial_parent_at_zoom(&self, target_z: u8) -> Result<RangeId, Error> {
        let z = self.z.get();
        if target_z > z {
            return Err(SpatialIdError::ZoomLevelTransitionOutOfRange {
                current_z: z,
                target_z,
            }
            .into());
        }

        if ZoomLevel::new(target_z).is_err() {
            return Err(SpatialIdError::ZOutOfRange { z: target_z }.into());
        }

        let shift = (z - target_z) as u32;

        let f = [
            if self.f[0] == -1 {
                -1
            } else {
                self.f[0] >> shift
            },
            if self.f[1] == -1 {
                -1
            } else {
                self.f[1] >> shift
            },
        ];

        let x = [self.x[0] >> shift, self.x[1] >> shift];
        let y = [self.y[0] >> shift, self.y[1] >> shift];

        Ok(RangeId {
            z: unsafe { ZoomLevel::new_unchecked(target_z) },
            f,
            x,
            y,

            temporal_id: self.temporal().clone(),
        })
    }
}
