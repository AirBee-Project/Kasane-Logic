use crate::{Error, FlexId, TemporalId, spatial_id::zoom_level::ZoomLevel};

impl FlexId {
    pub fn new(
        f_zoomlevel: impl Into<u8>,
        f_index: i32,
        x_zoomlevel: impl Into<u8>,
        x_index: u32,
        y_zoomlevel: impl Into<u8>,
        y_index: u32,
    ) -> Result<FlexId, Error> {
        let fz = ZoomLevel::new(f_zoomlevel.into())?;
        let xz = ZoomLevel::new(x_zoomlevel.into())?;
        let yz = ZoomLevel::new(y_zoomlevel.into())?;

        fz.check_f(f_index)?;
        xz.check_x(x_index)?;
        yz.check_y(y_index)?;

        Ok(FlexId {
            f_zoomlevel: fz,
            f_index,
            x_zoomlevel: xz,
            x_index,
            y_zoomlevel: yz,
            y_index,
            temporal_id: TemporalId::WHOLE,
        })
    }

    /// # Safety
    /// 呼び出し側は、各次元のズームレベルとインデックスが対応する有効範囲内であることを保証しなければなりません。
    pub unsafe fn new_unchecked(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
    ) -> FlexId {
        FlexId {
            f_zoomlevel: unsafe { ZoomLevel::new_unchecked(f_zoomlevel) },
            f_index,
            x_zoomlevel: unsafe { ZoomLevel::new_unchecked(x_zoomlevel) },
            x_index,
            y_zoomlevel: unsafe { ZoomLevel::new_unchecked(y_zoomlevel) },
            y_index,
            temporal_id: TemporalId::WHOLE,
        }
    }

    /// # Errors
    /// `f_zoomlevel`、`x_zoomlevel`、`y_zoomlevel` のいずれかが許容範囲外の場合は
    /// `SpatialIdError::ZOutOfRange` を返します。
    ///
    /// `f_index` が `f_zoomlevel` に対応する許容範囲外の場合は
    /// `SpatialIdError::FOutOfRange` を返します。
    ///
    /// `x_index` が `x_zoomlevel` に対応する許容範囲外の場合は
    /// `SpatialIdError::XOutOfRange` を返します。
    ///
    /// `y_index` が `y_zoomlevel` に対応する許容範囲外の場合は
    /// `SpatialIdError::YOutOfRange` を返します。
    ///
    /// `temporal_id` feature が無効な場合、`temporal_id` は常に全時間を表す
    /// スタブ（[`TemporalId::WHOLE`]）である。
    pub fn new_with_temporal(
        f_zoomlevel: impl Into<u8>,
        f_index: i32,
        x_zoomlevel: impl Into<u8>,
        x_index: u32,
        y_zoomlevel: impl Into<u8>,
        y_index: u32,
        temporal_id: TemporalId,
    ) -> Result<FlexId, Error> {
        let fz = ZoomLevel::new(f_zoomlevel.into())?;
        let xz = ZoomLevel::new(x_zoomlevel.into())?;
        let yz = ZoomLevel::new(y_zoomlevel.into())?;

        fz.check_f(f_index)?;
        xz.check_x(x_index)?;
        yz.check_y(y_index)?;

        Ok(FlexId {
            f_zoomlevel: fz,
            f_index,
            x_zoomlevel: xz,
            x_index,
            y_zoomlevel: yz,
            y_index,

            temporal_id,
        })
    }

    /// # Safety
    /// 呼び出し側は、各次元のズームレベルとインデックスが対応する有効範囲内であること、および `temporal_id` が有効な値であることを保証しなければなりません。
    pub unsafe fn new_with_temporal_unchecked(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
        temporal_id: TemporalId,
    ) -> FlexId {
        FlexId {
            f_zoomlevel: unsafe { ZoomLevel::new_unchecked(f_zoomlevel) },
            f_index,
            x_zoomlevel: unsafe { ZoomLevel::new_unchecked(x_zoomlevel) },
            x_index,
            y_zoomlevel: unsafe { ZoomLevel::new_unchecked(y_zoomlevel) },
            y_index,
            temporal_id,
        }
    }
}
