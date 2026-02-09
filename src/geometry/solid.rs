use crate::{Coordinate, Error, Polygon};
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// 隙間や穴のない、完全に閉じた立体を表す型。
///
/// 作成時に下記のことを完全に保証する。
/// - 立体に穴や隙間が一切空いていないこと。
/// - すべての辺（は、必ず他の面と隙間なく接合されていること。
/// - 1つの辺を3枚以上の面が共有したり、自己交差したりする構造ではないこと。
/// - すべての辺は、正確に2つの面によって共有されること。
/// - 頂点の座標はビット単位で厳密に一致していること。
pub struct Solid {
    surfaces: Vec<Polygon>,
}

impl Solid {
    pub fn new(surfaces: Vec<Polygon>) -> Result<Self, Error> {
        //面があることを確認する
        if surfaces.is_empty() {
            return Err(Error::EmptySolid);
        }

        let solid = Self { surfaces };

        solid.is_close()?;

        Ok(solid)
    }

    /// 立体が閉じていることを確認する
    fn is_close(&self) -> Result<(), Error> {
        // 座標をビット列に変換してハッシュマップのキーにする
        // 浮動小数点はEqが実装されていないため、bit列に変換する
        let to_bits = |c: &Coordinate| -> [u64; 3] {
            [
                c.as_latitude().to_bits(),
                c.as_longitude().to_bits(),
                c.as_altitude().to_bits(),
            ]
        };

        #[derive(Debug, Default)]
        struct EdgeStats {
            forward_count: usize,  // A -> B の出現回数
            backward_count: usize, // B -> A の出現回数
        }

        let mut edge_map: HashMap<([u64; 3], [u64; 3]), EdgeStats> = HashMap::new();

        for (surface_idx, surface) in self.surfaces.iter().enumerate() {
            let points = surface.points();

            for i in 0..points.len() - 1 {
                let p1_bits = to_bits(&points[i]);
                let p2_bits = to_bits(&points[i + 1]);

                if p1_bits == p2_bits {
                    return Err(Error::DegenerateEdge(surface_idx));
                }

                let key = if p1_bits < p2_bits {
                    (p1_bits, p2_bits)
                } else {
                    (p2_bits, p1_bits)
                };

                let stats = edge_map.entry(key).or_default();

                if p1_bits < p2_bits {
                    stats.forward_count += 1;
                } else {
                    stats.backward_count += 1;
                }
            }
        }

        //全体を検証する
        for (_, stats) in edge_map {
            // 穴が開いている
            if stats.forward_count == 0 || stats.backward_count == 0 {
                return Err(Error::OpenHoleDetected);
            }

            // 面の向きが不整合である
            if stats.forward_count > 1 || stats.backward_count > 1 {
                return Err(Error::NonManifoldEdge);
            }
        }

        Ok(())
    }

    ///立体をSingleIdの集合に変換する関数
    pub fn single_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = SingleId>, Error>
    {
        todo!()
    }

    ///立体をRangeIdの集合に変換する関数
    pub fn range_ids(&self, z: u8)
    //-> Result<impl Iterator<Item = RangeId>, Error>
    {
        todo!()
    }
}
