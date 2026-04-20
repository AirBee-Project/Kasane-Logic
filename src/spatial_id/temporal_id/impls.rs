use std::{fmt::Display, str::FromStr};

use crate::{SpatialIdError, TemporalId, error::Error, spatial_id::helpers::format_dimension};

impl Display for TemporalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", self.i)?;
        write!(f, "{}", format_dimension(self.t))?;
        Ok(())
    }
}

/// 文字列表現から [`TemporalId`] を復元します。
///
/// `temporal_id` feature が有効な場合は `"i/start:end"` または
/// `"i/value"` 形式を受け付けます。
///
/// ```
/// # use kasane_logic::TemporalId;
/// let temporal = TemporalId::new(60, [120, 179]).unwrap();
/// let parsed: TemporalId = temporal.to_string().parse().unwrap();
/// assert_eq!(parsed, temporal);
/// ```
impl FromStr for TemporalId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (i_str, t_str) = s.split_once('/').ok_or_else(|| {
            SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            }
            .into()
        })?;

        let i = i_str.parse::<u64>().map_err(|_| {
            SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            }
            .into()
        })?;

        let (start_str, end_str) = match t_str.split_once(':') {
            Some((start, end)) => (start, end),
            None => (t_str, t_str),
        };

        let start = start_str.parse::<u64>().map_err(|_| {
            SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            }
            .into()
        })?;
        let end = end_str.parse::<u64>().map_err(|_| {
            SpatialIdError::ParseSpatialIdFormat {
                kind: "TemporalId",
                input: s.to_string(),
            }
            .into()
        })?;

        TemporalId::new(i, [start, end])
    }
}
