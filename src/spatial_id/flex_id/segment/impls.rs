use std::fmt::{self, Display};

use crate::Segment;

impl<const N: usize> Display for Segment<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, byte) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:08b}", byte)?;
        }
        Ok(())
    }
}

impl<const N: usize> From<[u8; N]> for Segment<N> {
    fn from(value: [u8; N]) -> Self {
        Segment::new(value)
    }
}

impl<const N: usize> From<Segment<N>> for [u8; N] {
    fn from(value: Segment<N>) -> Self {
        value.0
    }
}
