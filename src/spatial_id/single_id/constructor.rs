#[allow(unused_imports)]
use alloc::boxed::Box;
#[allow(unused_imports)]
use alloc::rc::Rc;
#[allow(unused_imports)]
use alloc::string::{String, ToString};
#[allow(unused_imports)]
use alloc::vec::Vec;

use crate::SingleId;

use crate::{
    SpatialIdError, TemporalId,
    error::Error,
    spatial_id::constants::{F_MAX, F_MIN, MAX_ZOOM_LEVEL, XY_MAX},
};

impl SingleId {
    /// 指定された値から [`SingleId`] を作成する。このコンストラクタは、与えられた `z`, `f`, `x`, `y` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–[MAX_ZOOM_LEVEL]の範囲が有効）
    /// * `f` — Fインデックス（鉛直方向）
    /// * `x` — Xインデックス（東西方向）
    /// * `y` — Yインデックス（南北方向）
    ///
    /// # バリデーション
    /// - `z` が [`MAX_ZOOM_LEVEL`] を超える場合、[`SpatialIdError::ZOutOfRange`] を返す。
    /// - `f` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    /// - `x` または `y` が `0..=XY_MAX[z]` の範囲外の場合、
    ///   それぞれ [`SpatialIdError::XOutOfRange`]、[`SpatialIdError::YOutOfRange`] を返す。
    ///
    ///
    /// IDの作成:
    /// ```no_run
    /// # use kasane_logic::SingleId;
    /// let id = SingleId::new(5, 3, 2, 10).unwrap();
    /// assert_eq!(id.to_string(), "5/3/2/10".to_string());
    /// ```
    ///
    /// 次元の範囲外の検知:
    /// ```no_run
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialIdError;
    /// let id = SingleId::new(3, 3, 2, 10);
    /// assert_eq!(id, Err(SpatialIdError::YOutOfRange{z:3,y:10}.into()));
    /// ```
    ///
    /// ズームレベルの範囲外の検知:
    /// ```
    /// # use kasane_logic::SingleId;
    /// # use kasane_logic::SpatialIdError;
    /// let id = SingleId::new(68, 3, 2, 10);
    /// assert_eq!(id, Err(SpatialIdError::ZOutOfRange { z:68 }.into()));
    /// ```
    pub fn new(z: u8, f: i32, x: u32, y: u32) -> Result<SingleId, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f < f_min || f > f_max {
            return Err(SpatialIdError::FOutOfRange { f, z }.into());
        }
        if x > xy_max {
            return Err(SpatialIdError::XOutOfRange { x, z }.into());
        }
        if y > xy_max {
            return Err(SpatialIdError::YOutOfRange { y, z }.into());
        }

        Ok(SingleId {
            z,
            f,
            x,
            y,
            temporal_id: TemporalId::WHOLE,
        })
    }

    /// 検証を行わずに [`SingleId`] を作成する。
    ///
    /// この関数は [`SingleId::new`] と異なり、与えられた `z`, `f`, `x`, `y` に対して一切の範囲チェックや整合性チェックを行わない。そのため、高速に ID を生成できるが、**不正なパラメータを与えた場合の動作は未定義である**。
    ///
    /// # 注意
    /// 呼び出し側は、以下をすべて満たすことを保証しなければならない。
    ///
    /// * `z` が有効なズームレベル（0–[MAX_ZOOM_LEVEL]）であること
    /// * `f` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること
    /// * `x` および `y` が `0..=XY_MAX[z]` の範囲内であること
    ///
    /// これらが保証されない場合、パニック・不正メモリアクセス・未定義動作を引き起こす可能性がある。
    ///
    /// ```
    /// # use kasane_logic::SingleId;
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { SingleId::new_unchecked(5, 3, 2, 10) };
    ///
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.f(), 3i32);
    /// assert_eq!(id.x(), 2u32);
    /// assert_eq!(id.y(), 10u32);
    /// ```
    ///
    /// # Safety
    /// 呼び出し側は、`z` / `f` / `x` / `y` が各ズームレベルの有効範囲内であることを保証しなければなりません。
    pub unsafe fn new_unchecked(z: u8, f: i32, x: u32, y: u32) -> SingleId {
        SingleId {
            z,
            f,
            x,
            y,
            temporal_id: TemporalId::WHOLE,
        }
    }

    /// 指定された値から時間情報を指定した [`SingleId`] を作成する。このコンストラクタは、与えられた `z`, `f`, `x`, `y` が  各ズームレベルにおける範囲内にあるかを検証し、範囲外の場合は [`Error`] を返す。
    ///
    /// # パラメータ
    /// * `z` — ズームレベル（0–[MAX_ZOOM_LEVEL]の範囲が有効）
    /// * `f` — Fインデックス（鉛直方向）
    /// * `x` — Xインデックス（東西方向）
    /// * `y` — Yインデックス（南北方向）
    /// * `temporal_id` — [TemporalId](時間ID)
    ///
    /// # バリデーション
    /// - `z` が [`MAX_ZOOM_LEVEL`] を超える場合、[`SpatialIdError::ZOutOfRange`] を返す。
    /// - `f` がズームレベル `z` に対する `F_MIN[z]..=F_MAX[z]` の範囲外の場合、
    ///   [`SpatialIdError::FOutOfRange`] を返す。
    /// - `x` または `y` が `0..=XY_MAX[z]` の範囲外の場合、
    ///   それぞれ [`SpatialIdError::XOutOfRange`]、[`SpatialIdError::YOutOfRange`] を返す。
    ///
    ///
    /// IDの作成:
    /// ```no_run
    /// # use kasane_logic::{SingleId,TemporalId};
    /// //時間IDの作成
    /// let temporal_id = TemporalId::new(60, 1).unwrap();
    ///
    /// let id = SingleId::new_with_temporal(5, 3, 2, 10,temporal_id).unwrap();
    /// assert_eq!(id.to_string(), "5/3/2/10_60/1".to_string());
    /// ```
    #[cfg(feature = "temporal_id")]
    pub fn new_with_temporal(
        z: u8,
        f: i32,
        x: u32,
        y: u32,
        temporal_id: TemporalId,
    ) -> Result<SingleId, Error> {
        if z > MAX_ZOOM_LEVEL as u8 {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_min = F_MIN[z as usize];
        let f_max = F_MAX[z as usize];
        let xy_max = XY_MAX[z as usize];

        if f < f_min || f > f_max {
            return Err(SpatialIdError::FOutOfRange { f, z }.into());
        }
        if x > xy_max {
            return Err(SpatialIdError::XOutOfRange { x, z }.into());
        }
        if y > xy_max {
            return Err(SpatialIdError::YOutOfRange { y, z }.into());
        }

        Ok(SingleId {
            z,
            f,
            x,
            y,

            temporal_id,
        })
    }

    /// 検証を行わずに 時間情報を指定した[`SingleId`] を作成する。
    ///
    /// この関数は [`SingleId::new`] と異なり、与えられた `z`, `f`, `x`, `y` に対して一切の範囲チェックや整合性チェックを行わない。そのため、高速に ID を生成できるが、**不正なパラメータを与えた場合の動作は未定義である**。
    ///
    /// # 注意
    /// 呼び出し側は、以下をすべて満たすことを保証しなければならない。
    ///
    /// * `z` が有効なズームレベル（0–[MAX_ZOOM_LEVEL]）であること
    /// * `f` が与えられた `z` に応じて `F_MIN[z]..=F_MAX[z]` の範囲内であること
    /// * `x` および `y` が `0..=XY_MAX[z]` の範囲内であること
    ///
    /// これらが保証されない場合、パニック・不正メモリアクセス・未定義動作を引き起こす可能性がある。
    ///
    /// ```
    /// # use kasane_logic::{SingleId,TemporalId};
    /// //時間IDの作成
    /// let temporal_id = TemporalId::new(60, 1).unwrap();
    ///
    /// // パラメータが妥当であることを呼び出し側が保証する必要がある
    /// let id = unsafe { SingleId::new_with_temporal_unchecked(5, 3, 2, 10,temporal_id) };
    ///
    /// assert_eq!(id.z(), 5u8);
    /// assert_eq!(id.f(), 3i32);
    /// assert_eq!(id.x(), 2u32);
    /// assert_eq!(id.y(), 10u32);
    /// ```
    ///
    /// # Safety
    /// 呼び出し側は、`z` / `f` / `x` / `y` が各ズームレベルの有効範囲内であることに加え、`temporal_id` が有効な値であることを保証しなければなりません。
    #[cfg(feature = "temporal_id")]
    pub unsafe fn new_with_temporal_unchecked(
        z: u8,
        f: i32,
        x: u32,
        y: u32,
        temporal_id: TemporalId,
    ) -> SingleId {
        SingleId {
            z,
            f,
            x,
            y,
            temporal_id,
        }
    }
}
