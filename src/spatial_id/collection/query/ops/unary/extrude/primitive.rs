#[cfg(feature = "temporal_id")]
use crate::SpatialId;
use crate::{
    Error, FlexId, SpatialIdError, ZoomLevel,
    spatial_id::range_id::convert::{split_f, split_xy},
};
use alloc::vec::Vec;

impl FlexId {
    /// このFlexIdのF方向の占有を、ズーム `z` の絶対座標範囲 `[start_f, end_f]` に置き換える。
    pub fn extrude_f(
        &self,
        z: u8,
        start_f: i32,
        end_f: i32,
    ) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let (left, right) = (start_f.min(end_f), start_f.max(end_f));
        if left < ZoomLevel::new(z).unwrap().f_min() {
            return Err(SpatialIdError::FOutOfRange { z, f: left }.into());
        }
        if right > ZoomLevel::new(z).unwrap().f_max() {
            return Err(SpatialIdError::FOutOfRange { z, f: right }.into());
        }

        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal().clone();

        Ok(split_f(z, [left, right]).map(move |(seg_z, seg_index)| {
            #[cfg(feature = "temporal_id")]
            {
                FlexId::new_with_temporal(
                    seg_z,
                    seg_index,
                    x_zoomlevel,
                    x_index,
                    y_zoomlevel,
                    y_index,
                    temporal_id.clone(),
                )
                .unwrap()
            }

            #[cfg(not(feature = "temporal_id"))]
            {
                FlexId::new(seg_z, seg_index, x_zoomlevel, x_index, y_zoomlevel, y_index).unwrap()
            }
        }))
    }

    /// このFlexIdのX方向の占有を、ズーム `z` の絶対座標範囲（`start_x` から東向きに `end_x` まで）へ置き換える。
    pub fn extrude_x(
        &self,
        z: u8,
        start_x: u32,
        end_x: u32,
    ) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let xy_max = ZoomLevel::new(z).unwrap().xy_max();
        if start_x > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: start_x }.into());
        }
        if end_x > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: end_x }.into());
        }

        // start_x から東向きに end_x まで。境界跨ぎ（start_x > end_x）は2区間へ分ける。
        let ranges: Vec<[u32; 2]> = if start_x <= end_x {
            vec![[start_x, end_x]]
        } else {
            vec![[start_x, xy_max], [0, end_x]]
        };
        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let y_zoomlevel = self.y_zoomlevel();
        let y_index = self.y_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal().clone();

        Ok(ranges
            .into_iter()
            .flat_map(move |range| split_xy(z, range))
            .map(move |(seg_z, seg_index)| {
                #[cfg(feature = "temporal_id")]
                {
                    FlexId::new_with_temporal(
                        f_zoomlevel,
                        f_index,
                        seg_z,
                        seg_index,
                        y_zoomlevel,
                        y_index,
                        temporal_id.clone(),
                    )
                    .unwrap()
                }

                #[cfg(not(feature = "temporal_id"))]
                {
                    FlexId::new(f_zoomlevel, f_index, seg_z, seg_index, y_zoomlevel, y_index)
                        .unwrap()
                }
            }))
    }

    /// このFlexIdのY方向の占有を、ズーム `z` の絶対座標範囲 `[start_y, end_y]` に置き換える。
    pub fn extrude_y(
        &self,
        z: u8,
        start_y: u32,
        end_y: u32,
    ) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let (left, right) = (start_y.min(end_y), start_y.max(end_y));
        if right > ZoomLevel::new(z).unwrap().xy_max() {
            return Err(SpatialIdError::YOutOfRange { z, y: right }.into());
        }

        let f_zoomlevel = self.f_zoomlevel();
        let f_index = self.f_index();
        let x_zoomlevel = self.x_zoomlevel();
        let x_index = self.x_index();
        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal().clone();

        Ok(split_xy(z, [left, right]).map(move |(seg_z, seg_index)| {
            #[cfg(feature = "temporal_id")]
            {
                FlexId::new_with_temporal(
                    f_zoomlevel,
                    f_index,
                    x_zoomlevel,
                    x_index,
                    seg_z,
                    seg_index,
                    temporal_id.clone(),
                )
                .unwrap()
            }

            #[cfg(not(feature = "temporal_id"))]
            {
                FlexId::new(f_zoomlevel, f_index, x_zoomlevel, x_index, seg_z, seg_index).unwrap()
            }
        }))
    }

    /// このFlexIdのF, X, Y方向の占有を、ズーム `z` の絶対座標範囲に置き換える。
    #[allow(clippy::too_many_arguments)]
    pub fn extrude_fxy(
        &self,
        z: u8,
        start_f: i32,
        end_f: i32,
        start_x: u32,
        end_x: u32,
        start_y: u32,
        end_y: u32,
    ) -> Result<impl Iterator<Item = FlexId>, Error> {
        if z > ZoomLevel::MAX.get() {
            return Err(SpatialIdError::ZOutOfRange { z }.into());
        }

        let f_ranges = {
            let (left, right) = (start_f.min(end_f), start_f.max(end_f));
            if left < ZoomLevel::new(z).unwrap().f_min() {
                return Err(SpatialIdError::FOutOfRange { z, f: left }.into());
            }
            if right > ZoomLevel::new(z).unwrap().f_max() {
                return Err(SpatialIdError::FOutOfRange { z, f: right }.into());
            }
            vec![[left, right]]
        };

        let xy_max = ZoomLevel::new(z).unwrap().xy_max();
        if start_x > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: start_x }.into());
        }
        if end_x > xy_max {
            return Err(SpatialIdError::XOutOfRange { z, x: end_x }.into());
        }
        let x_ranges: Vec<[u32; 2]> = if start_x <= end_x {
            vec![[start_x, end_x]]
        } else {
            vec![[start_x, xy_max], [0, end_x]]
        };

        let y_ranges = {
            let (left, right) = (start_y.min(end_y), start_y.max(end_y));
            if right > xy_max {
                return Err(SpatialIdError::YOutOfRange { z, y: right }.into());
            }
            vec![[left, right]]
        };

        #[cfg(feature = "temporal_id")]
        let temporal_id = self.temporal().clone();

        let mut out = Vec::new();
        for f_r in f_ranges {
            for (seg_fz, seg_fi) in split_f(z, f_r) {
                for x_r in &x_ranges {
                    for (seg_xz, seg_xi) in split_xy(z, *x_r) {
                        for y_r in &y_ranges {
                            for (seg_yz, seg_yi) in split_xy(z, *y_r) {
                                #[cfg(feature = "temporal_id")]
                                {
                                    out.push(
                                        FlexId::new_with_temporal(
                                            seg_fz,
                                            seg_fi,
                                            seg_xz,
                                            seg_xi,
                                            seg_yz,
                                            seg_yi,
                                            temporal_id.clone(),
                                        )
                                        .unwrap(),
                                    );
                                }
                                #[cfg(not(feature = "temporal_id"))]
                                {
                                    out.push(
                                        FlexId::new(seg_fz, seg_fi, seg_xz, seg_xi, seg_yz, seg_yi)
                                            .unwrap(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(out.into_iter())
    }
}
