use alloc::string::ToString;

use core::{fmt::Display, str::FromStr};

use crate::{SpatialIdError, TemporalCell, error::Error};

impl Display for TemporalCell {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}/{}", self.zoom(), self.index())
    }
}

impl Default for TemporalCell {
    fn default() -> Self {
        Self {
            zoom: unsafe { crate::spatial_id::temporal_zoom_level::TZoomLevel::new_unchecked(0) },
            index: 0,
        }
    }
}

/// 文字列表現から [`TemporalCell`] を復元する。
///
/// `"zoom/index"` 形式の文字列をパースして [`TemporalCell`] を構築する。
///
/// ```
/// # #[cfg(feature = "temporal_id")]
/// # {
/// # use kasane_logic::TemporalCell;
/// # use core::str::FromStr;
/// let id = TemporalCell::new(5, 3).unwrap();
/// let parsed: TemporalCell = "5/3".parse().unwrap();
/// assert_eq!(id, parsed);
/// # }
/// ```
impl FromStr for TemporalCell {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (zoom_text, index_text) =
            s.split_once('/')
                .ok_or_else(|| SpatialIdError::ParseSpatialIdFormat {
                    kind: "TemporalCell",
                    input: s.to_string(),
                })?;

        let zoom = zoom_text
            .parse::<u8>()
            .map_err(|_| SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalCell",
                input: s.to_string(),
            })?;

        let index =
            index_text
                .parse::<u64>()
                .map_err(|_| SpatialIdError::ParseSpatialIdFormat {
                    kind: "TemporalCell",
                    input: s.to_string(),
                })?;

        TemporalCell::new(zoom, index)
    }
}
