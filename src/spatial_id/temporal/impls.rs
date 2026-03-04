use std::fmt::Display;

use crate::TemporalId;

impl Display for TemporalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/", self.i)?;

        if self.t[0] == self.t[1] {
            write!(f, "{}", self.t[0])?;
        } else {
            write!(f, "{}:{}", self.t[0], self.t[1])?;
        }

        Ok(())
    }
}
