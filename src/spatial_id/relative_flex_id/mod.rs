use crate::{
    Error, FlexId, SpatialIdError,
    spatial_id::{
        dimension::Dimension,
        zoom_level::{TZoomLevel, ZoomLevel},
    },
};

/// ある祖先の [FlexId] を起点とした相対的な位置を表す[FlexID]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct RelativeFlexId(FlexId);

impl FlexId {
    /// `ancestor` を原点とした[RelativeFlexId]を返す。
    ///
    /// `ancestor`が完全に自身を包含していなければ [`SpatialIdError::NotAncestor`] を返す。
    pub(crate) fn relative_to(&self, ancestor: &FlexId) -> Result<RelativeFlexId, Error> {
        if !ancestor.contains(self) {
            return Err(SpatialIdError::NotAncestor.into());
        }
        // 各軸で、祖先からの深さと、その深さぶんの下位ビットを求める
        let relative = from_axes(zip_axes(self, ancestor, |(z, i), (az, ai)| {
            let dz = z - az;
            (dz, i - (ai << dz))
        }))?;
        Ok(RelativeFlexId(relative))
    }
}

impl RelativeFlexId {
    /// 祖先から `dimension`において何段深いかを返す。
    pub(crate) fn depth_on(self, dimension: Dimension) -> u8 {
        self.0.zoomlevel_on(dimension)
    }

    /// 祖先より深くなっている次元の集合（[`Dimension::bit`] の OR）。
    pub(crate) fn deeper_dimensions(&self) -> u8 {
        Dimension::mask(|d| self.depth_on(d) > 0)
    }

    /// `ancestor` を原点として、[FlexId] を復元する。
    /// どれかの軸で最大ズームを超えるなら [`SpatialIdError::ZOutOfRange`] を返す。
    pub(crate) fn to_absolute(self, ancestor: &FlexId) -> Result<FlexId, Error> {
        let max_zoom = [
            ZoomLevel::MAX.get(),
            ZoomLevel::MAX.get(),
            ZoomLevel::MAX.get(),
            TZoomLevel::MAX.get(),
        ];
        for (((az, _), (rz, _)), max) in axes(ancestor).into_iter().zip(axes(&self.0)).zip(max_zoom)
        {
            if az + rz > max {
                return Err(SpatialIdError::ZOutOfRange { z: az + rz }.into());
            }
        }
        from_axes(zip_axes(ancestor, &self.0, |(az, ai), (rz, ri)| {
            (az + rz, (ai << rz) + ri)
        }))
    }
}

/// F / X / Y / T の各軸の `(ズーム, インデックス)`。
type Axes = [(u8, i64); 4];

fn axes(id: &FlexId) -> Axes {
    [
        (id.f_zoomlevel(), id.f_index() as i64),
        (id.x_zoomlevel(), id.x_index() as i64),
        (id.y_zoomlevel(), id.y_index() as i64),
        (id.t_zoomlevel(), id.t() as i64),
    ]
}

/// 2つの ID の各軸を `f` で組み合わせる。
fn zip_axes(a: &FlexId, b: &FlexId, f: impl Fn((u8, i64), (u8, i64)) -> (u8, i64)) -> Axes {
    let (a, b) = (axes(a), axes(b));
    core::array::from_fn(|i| f(a[i], b[i]))
}

fn from_axes([(fz, f), (xz, x), (yz, y), (tz, t)]: Axes) -> Result<FlexId, Error> {
    Ok(FlexId::new(fz, f as i32, xz, x as u32, yz, y as u32)?.with_time_segment(tz, t as u64))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 自身を包含しない ID を祖先に渡すとエラーになる。
    #[test]
    fn relative_to_non_ancestor_is_error() {
        let id = FlexId::new(3, 1, 3, 2, 3, 3).unwrap();
        let sibling = FlexId::new(3, 0, 3, 2, 3, 3).unwrap();
        let child = FlexId::new(4, 2, 3, 2, 3, 3).unwrap();
        assert_eq!(
            id.relative_to(&sibling),
            Err(SpatialIdError::NotAncestor.into())
        );
        assert!(id.relative_to(&child).is_err());
    }

    /// 相対 ID は、作ったときの祖先から絶対 ID へ戻せる。
    #[test]
    fn to_absolute_restores_original() {
        let ancestor = FlexId::new(2, 1, 1, 0, 3, 5).unwrap();
        let id = FlexId::new(5, 12, 4, 7, 3, 5).unwrap();
        let relative = id.relative_to(&ancestor).unwrap();
        assert_eq!(relative.to_absolute(&ancestor), Ok(id));
    }

    /// 深い祖先に当てはめて最大ズームを超えるならエラーになる。
    #[test]
    fn to_absolute_beyond_max_zoom_is_error() {
        let deep = FlexId::new(30, 0, 0, 0, 0, 0).unwrap();
        let relative = deep.relative_to(&FlexId::UPPER_MAX).unwrap();
        assert_eq!(
            relative.to_absolute(&deep),
            Err(SpatialIdError::ZOutOfRange { z: 60 }.into())
        );
    }
}
