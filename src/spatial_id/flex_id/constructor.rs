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


}
