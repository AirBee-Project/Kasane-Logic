use core::fmt;

use crate::bit_vec::BitVec;

impl fmt::Display for BitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ebit in &self.0 {
            write!(f, "{:08b}", ebit)?;
        }
        Ok(())
    }
}
