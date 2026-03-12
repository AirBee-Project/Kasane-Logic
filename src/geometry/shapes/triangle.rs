use std::collections::HashSet;

use crate::{Coordinate, Ecef, Error, SingleId};

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
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();
        let p2: Ecef = self.points[2].into();
        let a = [p1.x() - p0.x(), p1.y() - p0.y(), p1.z() - p0.z()];
        let b = [p2.x() - p0.x(), p2.y() - p0.y(), p2.z() - p0.z()];
        ((a[0] * a[0] + a[1] * a[1] + a[2] * a[2]) * (b[0] * b[0] + b[1] * b[1] + b[2] * b[2])
            - dot_product(a, b) * dot_product(a, b))
        .sqrt()
            * 0.5
    }

    ///三角形の3辺の長さを返す
    /// point[0]と[1],[1]と[2],[2]と[0]の順に返す
    pub fn sides(&self) -> [f64; 3] {
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();
        let p2: Ecef = self.points[2].into();
        [p0.distance(&p1), p1.distance(&p2), p2.distance(&p0)]
    }

    ///[SingleId]の集合へ変換を行います。
    pub fn divide(&self, steps: u32) -> Result<impl Iterator<Item = Triangle>, Error> {
        let steps_f = steps as f64;
        let p0: Ecef = self.points[0].into();
        let p1: Ecef = self.points[1].into();
        let p2: Ecef = self.points[2].into();

        let initial_row = vec![p0];

        let iter = (1..=steps)
            .scan(initial_row, move |prev_row, i| {
                let i_f = i as f64;

                let current_row: Vec<Ecef> = (0..=i)
                    .map(|j| {
                        let j_f = j as f64;
                        let w0 = 1.0 - (i_f / steps_f);
                        let w1 = (i_f - j_f) / steps_f;
                        let w2 = j_f / steps_f;

                        Ecef::new(
                            p0.x() * w0 + p1.x() * w1 + p2.x() * w2,
                            p0.y() * w0 + p1.y() * w1 + p2.y() * w2,
                            p0.z() * w0 + p1.z() * w1 + p2.z() * w2,
                        )
                    })
                    .collect();

                let mut row_triangles = Vec::with_capacity((i * 2 - 1) as usize);

                for j in 0..(i as usize) {
                    row_triangles.push(Triangle {
                        points: [
                            (*prev_row)[j].try_into().unwrap(),
                            current_row[j].try_into().unwrap(),
                            current_row[j + 1].try_into().unwrap(),
                        ],
                    });

                    if j > 0 {
                        row_triangles.push(Triangle {
                            points: [
                                (*prev_row)[j - 1].try_into().ok()?,
                                (*prev_row)[j].try_into().ok()?,
                                current_row[j].try_into().ok()?,
                            ],
                        });
                    }
                }

                *prev_row = current_row;

                Some(row_triangles)
            })
            .flat_map(|triangles| triangles.into_iter());
        Ok(iter)
    }

    fn single_ids_limited(self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [[f64; 3]; 3] = [
            coordinate_to_matrix(self.points[0], z),
            coordinate_to_matrix(self.points[1], z),
            coordinate_to_matrix(self.points[2], z),
        ];
        let mut voxels: HashSet<SingleId> = HashSet::new();
        let vec_a = [0, 1, 2].map(|i| points[1][i] - points[0][i]);
        let vec_b = [0, 1, 2].map(|i| points[0][i] - points[2][i]);
        let vec_c = [0, 1, 2].map(|i| points[2][i] - points[1][i]);
        let n = cross_product(vec_b, vec_a);
        let ma = cross_product(n, vec_a);
        let mb = cross_product(n, vec_b);
        let mc = cross_product(n, vec_c);
        let min_f = points[0][0].min(points[1][0]).min(points[2][0]).floor() as i32;
        let max_f = points[0][0].max(points[1][0]).max(points[2][0]).floor() as i32;
        let min_x = points[0][1].min(points[1][1]).min(points[2][1]).floor() as u32;
        let max_x = points[0][1].max(points[1][1]).max(points[2][1]).floor() as u32;
        let min_y = points[0][2].min(points[1][2]).min(points[2][2]).floor() as u32;
        let max_y = points[0][2].max(points[1][2]).max(points[2][2]).floor() as u32;
        let eight_patterns = [
            (0.0, 0.0, 0.0),
            (0.0, 0.0, 1.0),
            (0.0, 1.0, 0.0),
            (0.0, 1.0, 1.0),
            (1.0, 0.0, 0.0),
            (1.0, 0.0, 1.0),
            (1.0, 1.0, 0.0),
            (1.0, 1.0, 1.0),
        ];
        for f in min_f..=max_f {
            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let mut sign_before = true;
                    for i in 0..8 {
                        let vec_p = [
                            f as f64 + eight_patterns[i].0 - points[0][0],
                            x as f64 + eight_patterns[i].1 - points[0][1],
                            y as f64 + eight_patterns[i].2 - points[0][2],
                        ];
                        let sign = dot_product(n, vec_p).is_sign_positive();
                        if i == 0 || sign_before == sign {
                            sign_before = sign;
                        } else {
                            for i in 0..8 {
                                let cp = [
                                    f as f64 + eight_patterns[i].0,
                                    x as f64 + eight_patterns[i].1,
                                    y as f64 + eight_patterns[i].2,
                                ];
                                let rel_p0 = [
                                    cp[0] - points[0][0],
                                    cp[1] - points[0][1],
                                    cp[2] - points[0][2],
                                ];
                                let rel_p1 = [
                                    cp[0] - points[1][0],
                                    cp[1] - points[1][1],
                                    cp[2] - points[1][2],
                                ];
                                let rel_p2 = [
                                    cp[0] - points[2][0],
                                    cp[1] - points[2][1],
                                    cp[2] - points[2][2],
                                ];

                                if dot_product(ma, rel_p0) >= 0.0
                                    && dot_product(mc, rel_p1) >= 0.0
                                    && dot_product(mb, rel_p2) >= 0.0
                                {
                                    voxels.insert(unsafe { SingleId::new_unchecked(z, f, x, y) });
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

    pub fn single_ids(&self, z: u8) -> Result<impl Iterator<Item = SingleId>, Error> {
        let points: [[f64; 3]; 3] = [
            coordinate_to_matrix(self.points[0], z),
            coordinate_to_matrix(self.points[1], z),
            coordinate_to_matrix(self.points[2], z),
        ];
        let diff_f = points[0][0].max(points[1][0]).max(points[2][0]).floor()
            - points[0][0].min(points[1][0]).min(points[2][0]).floor();
        let diff_x = points[0][1].max(points[1][1]).max(points[2][1]).floor()
            - points[0][1].min(points[1][1]).min(points[2][1]).floor();
        let diff_y = points[0][2].max(points[1][2]).max(points[2][2]).floor()
            - points[0][2].min(points[1][2]).min(points[2][2]).floor();
        let steps = (diff_f.max(diff_x).max(diff_y) / 8.0).ceil() as u32;
        println!("{}", steps);
        let mut seen = HashSet::new();
        let voxels = self
            .divide(steps)?
            .flat_map(move |tri| tri.single_ids_limited(z).ok().into_iter().flatten())
            .filter(move |voxel| seen.insert(voxel.clone()));
        Ok(voxels)
    }
}

fn coordinate_to_matrix(p: Coordinate, z: u8) -> [f64; 3] {
    let lat = p.latitude();
    let lon = p.longitude();
    let alt = p.altitude();

    // 空間idの高さはz=25でちょうど1mになるように定義されている
    let factor = 2_f64.powi(z as i32 - 25);
    let f = factor * alt;

    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n;
    [f, x, y]
}

fn dot_product(vec_a: [f64; 3], vec_b: [f64; 3]) -> f64 {
    vec_a[0] * vec_b[0] + vec_a[1] * vec_b[1] + vec_a[2] * vec_b[2]
}
fn cross_product(vec_a: [f64; 3], vec_b: [f64; 3]) -> [f64; 3] {
    [
        vec_a[1] * vec_b[2] - vec_a[2] * vec_b[1],
        vec_a[2] * vec_b[0] - vec_a[0] * vec_b[2],
        vec_a[0] * vec_b[1] - vec_a[1] * vec_b[0],
    ]
}
