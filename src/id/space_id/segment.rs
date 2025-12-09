use crate::bit_vec::BitVec;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Segment<T> {
    pub z: u8,
    pub dim: T,
}

impl Segment<u64> {
    /// XY座標 Segment を BitVec へ変換
    pub fn to_bitvec(self) -> BitVec {
        let z = self.z;
        let xy = self.dim;

        let length = ((z * 2 / 8) + 1).max(1) as usize;
        let mut result = vec![0u8; length];

        let bit_count = (z + 1) as u32;
        let mask = if bit_count >= 64 {
            u64::MAX
        } else {
            (1u64 << bit_count) - 1
        };
        let uxy = xy & mask;

        for now_z in (0..=z).rev() {
            let index = ((now_z) * 2 / 8) as usize;
            let in_index = now_z % 4;

            result[index] |= 1 << (7 - in_index * 2);

            let bit_position = z - now_z;
            if (uxy >> bit_position) & 1 != 0 {
                result[index] |= 1 << (6 - in_index * 2);
            }
        }

        BitVec::from_vec(result)
    }

    /// XY向けのセグメント分割
    pub fn new(z: u8, dimension: [u64; 2]) -> Vec<Segment<u64>> {
        let mut target = dimension;
        let mut now_z = z;
        let mut result = Vec::new();

        loop {
            if target[0] == target[1] {
                result.push(Segment {
                    z: now_z,
                    dim: target[0],
                });
                break;
            }

            if target[0] % 2 != 0 {
                result.push(Segment {
                    z: now_z,
                    dim: target[0],
                });
                target[0] += 1;
            }

            if target[1] % 2 == 0 {
                result.push(Segment {
                    z: now_z,
                    dim: target[1],
                });
                target[1] -= 1;
            }

            if target[0] > target[1] {
                break;
            }

            if now_z == 0 {
                break;
            }

            target = [target[0] / 2, target[1] / 2];
            now_z -= 1;
        }

        result
    }
}

impl Segment<i64> {
    /// F座標 Segment を BitVec へ変換
    pub fn to_bitvec(self) -> BitVec {
        let z = self.z;
        let f = self.dim;

        if f >= 0 {
            return Segment { z, dim: f as u64 }.to_bitvec();
        }

        let u = (f.abs() - 1) as u64;
        let mut converted = Segment { z, dim: u }.to_bitvec();

        converted.0[0] |= 0b11000000;
        converted
    }

    /// F向けのセグメント分割
    pub fn new(z: u8, dimension: [i64; 2]) -> Vec<Segment<i64>> {
        let diff = 2_i64.pow(z.into());
        let mut target = [dimension[0] + diff, dimension[1] + diff];
        let mut now_z = z;
        let mut result = Vec::new();

        loop {
            if target[0] == target[1] {
                result.push(Segment {
                    z: now_z,
                    dim: target[0] - 2_i64.pow(now_z.into()),
                });
                break;
            }

            if target[0] % 2 != 0 {
                result.push(Segment {
                    z: now_z,
                    dim: target[0] - 2_i64.pow(now_z.into()),
                });
                target[0] += 1;
            }

            if target[1] % 2 == 0 {
                result.push(Segment {
                    z: now_z,
                    dim: target[1] - 2_i64.pow(now_z.into()),
                });
                target[1] -= 1;
            }

            if target[0] > target[1] {
                break;
            }

            if now_z == 0 {
                break;
            }

            target = [target[0] / 2, target[1] / 2];
            now_z -= 1;
        }

        result
    }
}
