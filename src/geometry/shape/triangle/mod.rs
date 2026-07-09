use alloc::vec::Vec;

use hashbrown::HashSet;

use crate::{Coordinate, Ecef, Error, SingleId, Vec3, Vec3Ecef, Vec3FractionalId};
pub mod geometry_relation;
pub mod impls;
#[cfg(test)]
mod tests;

///三角形を表す型
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub points: [Coordinate; 3],
}

impl Triangle {
    ///[Triangle]を作成する。
    ///
    /// 3つの点が一直線上にある場合や同一の座標の場合も問題なく作成される。
    pub fn new(points: [Coordinate; 3]) -> Self {
        Self { points }
    }

    ///三角形の面積を返す
    pub fn area(&self) -> f64 {
        let p0: Vec3Ecef = self.points[0].into();
        let p1: Vec3Ecef = self.points[1].into();
        let p2: Vec3Ecef = self.points[2].into();
        let a = p1 - p0;
        let b = p2 - p0;
        let dot = a.dot(&b);
        libm::sqrt(a.norm_squared() * b.norm_squared() - dot * dot) * 0.5
    }

    ///三角形の3辺の長さを返す
    /// point\[0\]と\[1\],\[1\]と\[2\],\[2\]と\[0\]の順に返す
    pub fn sides(&self) -> [f64; 3] {
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();
        let p2: Ecef = self.points[2].into();
        [p0.distance(&p1), p1.distance(&p2), p2.distance(&p0)]
    }

    pub fn divide(&self, steps: u32) -> Result<impl Iterator<Item = Triangle>, Error> {
        let steps_f = f64::from(steps);
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();
        let p2: Ecef = self.points[2].into();

        // [最適化1] 最初の頂点を1度だけ変換しておく
        let initial_pt: Coordinate = p0
            .try_into()
            .unwrap_or_else(|_| panic!("Failed to convert initial point"));

        // [最適化2] 毎行のアロケーションを避けるため、2つのバッファを用意して使い回す
        let initial_row = vec![initial_pt];
        let current_row_buf = Vec::with_capacity((steps + 1) as usize);

        // [最適化3] 行内で一定となるステップ値（P2 - P1）/ steps を事前計算
        let step_x = (p2.x() - p1.x()) / steps_f;
        let step_y = (p2.y() - p1.y()) / steps_f;
        let step_z = (p2.z() - p1.z()) / steps_f;

        let iter = (1..=steps)
            .scan(
                (initial_row, current_row_buf),
                move |(prev_row, current_row), i| {
                    let i_f = f64::from(i);

                    // バッファをクリアして再利用（メモリ割り当てが発生しない）
                    current_row.clear();

                    // [最適化3] 行の始点 (j = 0 のときの座標) を計算
                    let w0 = 1.0 - (i_f / steps_f);
                    let w1_base = i_f / steps_f;
                    let start_x = p0.x() * w0 + p1.x() * w1_base;
                    let start_y = p0.y() * w0 + p1.y() * w1_base;
                    let start_z = p0.z() * w0 + p1.z() * w1_base;

                    // [最適化1] 頂点の「生成時」にのみ1回だけ型変換を行う
                    for j in 0..=i {
                        let j_f = f64::from(j);
                        let ecef = Ecef::new(
                            start_x + step_x * j_f,
                            start_y + step_y * j_f,
                            start_z + step_z * j_f,
                        );

                        let pt: Coordinate = ecef.try_into().ok()?;
                        current_row.push(pt);
                    }

                    let mut row_triangles = Vec::with_capacity((i * 2 - 1) as usize);

                    // 変換済みの頂点を Copy で流用するため変換コストはゼロ
                    for j in 0..(i as usize) {
                        row_triangles.push(Triangle {
                            points: [prev_row[j], current_row[j], current_row[j + 1]],
                        });

                        if j > 0 {
                            row_triangles.push(Triangle {
                                points: [prev_row[j - 1], prev_row[j], current_row[j]],
                            });
                        }
                    }

                    // [最適化2] 次のイテレーションのためにバッファをスワップ
                    core::mem::swap(prev_row, current_row);

                    Some(row_triangles)
                },
            )
            .flat_map(core::iter::IntoIterator::into_iter);

        Ok(iter)
    }

    ///[`SingleId`]の集合へ変換を行います。
    pub fn single_ids_limited(self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [Vec3FractionalId; 3] = [
            Vec3FractionalId::from(self.points[0].fractional_id(z)?),
            Vec3FractionalId::from(self.points[1].fractional_id(z)?),
            Vec3FractionalId::from(self.points[2].fractional_id(z)?),
        ];
        let mut voxels: HashSet<SingleId> = HashSet::new();
        let vec_a = points[1] - points[0]; // 1-0
        let vec_b = points[0] - points[2]; // 0-2
        let vec_c = points[2] - points[1]; // 2-1
        let n = vec_b.cross(&vec_a);
        let ma = n.cross(&vec_a);
        let mb = n.cross(&vec_b);
        let mc = n.cross(&vec_c);
        let min_f = libm::floor(points[0].a().min(points[1].a()).min(points[2].a())) as i32;
        let max_f = libm::floor(points[0].a().max(points[1].a()).max(points[2].a())) as i32;
        let min_x = libm::floor(points[0].b().min(points[1].b()).min(points[2].b())) as u32;
        let max_x = libm::floor(points[0].b().max(points[1].b()).max(points[2].b())) as u32;
        let min_y = libm::floor(points[0].c().min(points[1].c()).min(points[2].c())) as u32;
        let max_y = libm::floor(points[0].c().max(points[1].c()).max(points[2].c())) as u32;
        let eight_patterns = [
            Vec3FractionalId::new(0.0, 0.0, 0.0),
            Vec3FractionalId::new(0.0, 0.0, 1.0),
            Vec3FractionalId::new(0.0, 1.0, 0.0),
            Vec3FractionalId::new(0.0, 1.0, 1.0),
            Vec3FractionalId::new(1.0, 0.0, 0.0),
            Vec3FractionalId::new(1.0, 0.0, 1.0),
            Vec3FractionalId::new(1.0, 1.0, 0.0),
            Vec3FractionalId::new(1.0, 1.0, 1.0),
        ];
        for f in min_f..=max_f {
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let mut sign_before = true;
                    for (i, pattern) in eight_patterns.iter().enumerate() {
                        let vec_p = Vec3FractionalId::new(
                            f64::from(f) + pattern.a() - points[0].a(),
                            f64::from(x) + pattern.b() - points[0].b(),
                            f64::from(y) + pattern.c() - points[0].c(),
                        );
                        let sign = n.dot(&vec_p).is_sign_positive();
                        if i == 0 || sign_before == sign {
                            sign_before = sign;
                        } else {
                            for pattern in eight_patterns {
                                let cp = Vec3FractionalId::new(
                                    f64::from(f) + pattern.a(),
                                    f64::from(x) + pattern.b(),
                                    f64::from(y) + pattern.c(),
                                );
                                let rel_p0 = cp - points[0];
                                let rel_p1 = cp - points[1];
                                let rel_p2 = cp - points[2];

                                if ma.dot(&rel_p0) >= 0.0
                                    && mc.dot(&rel_p1) >= 0.0
                                    && mb.dot(&rel_p2) >= 0.0
                                {
                                    voxels.insert(SingleId::new(z, f, x, y).unwrap());
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
        Ok(voxels.into_iter())
    }
}
