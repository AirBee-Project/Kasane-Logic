use core::fmt;

use crate::r#type::bit_vec::BitVec;

impl fmt::Display for BitVec {
    ///暫定的に全てを範囲記法にする
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();

        for byte in self.0 {
            for two_bit in 0..4 {
                let mask: u8 = 0b11000000;
            }
        }
    }
}
