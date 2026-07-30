use crate::{
    Error, FlexId,
    spatial_id::zoom_level::{TZoomLevel, ZoomLevel},
};

impl FlexId {
    /// 空間3軸から [`FlexId`] を構築する。時間軸は全時間になる。
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
            t_zoomlevel: TZoomLevel::MIN,
            t_index: 0,
        })
    }

    /// 4軸すべてを指定して [`FlexId`] を構築する。
    ///
    /// 時間軸も空間軸と同じ「ズームレベル＋インデックス」で指定する。1セルは
    /// `2^(35 - t_zoomlevel)` 秒を表す。
    ///
    /// # バリデーション
    /// - `t_zoomlevel` が `35` を超える場合は
    ///   [`SpatialIdError::ZOutOfRange`](crate::SpatialIdError::ZOutOfRange) を返す。
    /// - `t_index` が `2^t_zoomlevel - 1` を超える場合は
    ///   [`SpatialIdError::TOutOfRange`](crate::SpatialIdError::TOutOfRange) を返す。
    ///
    /// ```
    /// # #[cfg(feature = "temporal_id")]
    /// # {
    /// # use kasane_logic::FlexId;
    /// // 最深ズーム（35）は1秒幅のセル。
    /// let id = FlexId::new_with_time(5, 3, 2, 3, 10, 1, 35, 1_770_000_000).unwrap();
    /// assert_eq!(id.seconds_range(), (1_770_000_000, 1_770_000_001));
    /// assert_eq!(id.interval().seconds(), 1);
    /// # }
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_time(
        f_zoomlevel: impl Into<u8>,
        f_index: i32,
        x_zoomlevel: impl Into<u8>,
        x_index: u32,
        y_zoomlevel: impl Into<u8>,
        y_index: u32,
        t_zoomlevel: impl Into<u8>,
        t_index: u64,
    ) -> Result<FlexId, Error> {
        let base = FlexId::new(
            f_zoomlevel,
            f_index,
            x_zoomlevel,
            x_index,
            y_zoomlevel,
            y_index,
        )?;

        let tz = TZoomLevel::new(t_zoomlevel.into())?;
        tz.check_index(t_index)?;

        Ok(base.with_time_cell(tz.get(), t_index))
    }

    /// 検証を行わずに空間3軸から構築する。時間軸は全時間になる。
    ///
    /// # Safety
    /// 呼び出し側は、各次元のズームレベルとインデックスが対応する有効範囲内であることを保証しなければなりません。
    #[inline]
    pub unsafe fn new_unchecked(
        f_zoomlevel: u8,
        f_index: i32,
        x_zoomlevel: u8,
        x_index: u32,
        y_zoomlevel: u8,
        y_index: u32,
    ) -> FlexId {
        debug_assert!(
            FlexId::new(
                f_zoomlevel,
                f_index,
                x_zoomlevel,
                x_index,
                y_zoomlevel,
                y_index
            )
            .is_ok(),
            "new_unchecked: 構成的に有効なはずの FlexId が検証に失敗した \
             (zf={f_zoomlevel} f={f_index} zx={x_zoomlevel} x={x_index} zy={y_zoomlevel} y={y_index})"
        );
        FlexId {
            f_zoomlevel: unsafe { ZoomLevel::new_unchecked(f_zoomlevel) },
            f_index,
            x_zoomlevel: unsafe { ZoomLevel::new_unchecked(x_zoomlevel) },
            x_index,
            y_zoomlevel: unsafe { ZoomLevel::new_unchecked(y_zoomlevel) },
            y_index,
            t_zoomlevel: TZoomLevel::MIN,
            t_index: 0,
        }
    }
}
