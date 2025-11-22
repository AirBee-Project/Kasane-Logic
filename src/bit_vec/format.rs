use core::fmt;

use crate::bit_vec::HierarchicalKey;

impl fmt::Display for HierarchicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ebit in &self.0 {
            write!(f, "{:08b}", ebit)?;
        }
        Ok(())
    }
}
