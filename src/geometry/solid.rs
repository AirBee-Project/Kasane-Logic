use crate::{Coordinate, Error, Surface};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Solid {
    surfaces: Vec<Surface>,
}

impl Solid {
    pub fn new(surfaces: Vec<Surface>) -> Result<Self, Error> {
        if surfaces.is_empty() {
            return Err(Error::NoSurfaces);
        }

        let solid = Self { surfaces };

        // 閉じているか検証
        solid.validate_closure()?;

        Ok(solid)
    }

    /// Solidが閉じているか検証
    fn validate_closure(&self) -> Result<(), Error> {
        // Coordinate を [u64; 3] に変換する関数
        let to_key = |c: &Coordinate| -> [u64; 3] {
            [
                c.as_latitude().to_bits(),
                c.as_longitude().to_bits(),
                c.as_altitude().to_bits(),
            ]
        };

        let mut edge_counts: HashMap<([u64; 3], [u64; 3]), usize> = HashMap::new();

        for surface in &self.surfaces {
            let points = surface.points();

            // 閉じたリングなので、最後の点（= 最初の点）を除外
            // points.len() - 1 までループ
            for i in 0..points.len() - 1 {
                let p1 = &points[i];
                let p2 = &points[i + 1];

                let k1 = to_key(p1);
                let k2 = to_key(p2);

                // エッジを正規化（小さい方が先）
                let key = if k1 < k2 { (k1, k2) } else { (k2, k1) };

                *edge_counts.entry(key).or_insert(0) += 1;
            }
        }

        // 全てのエッジがちょうど2回使われているか検証
        for (_, count) in edge_counts {
            match count {
                1 => return Err(Error::NotClosedSolid),
                2 => continue, // OK
                _ => return Err(Error::NonManifoldSolid(count)),
            }
        }

        Ok(())
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn surface_count(&self) -> usize {
        self.surfaces.len()
    }
}
