use std::fmt::Display;

use crate::{TemporalId, spatial_id::helpers::format_dimension};

impl Display for TemporalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", self.i)?;
        write!(f, "{}", format_dimension(self.t))?;
        Ok(())
    }
}
