use std::f64::consts::PI;

use crate::{
    encode_id::EncodeID,
    encode_id_set::EncodeIDSet,
    point::{ECEF, Point},
    space_time_id::encode::to_bitvec::{to_bitvec_f, to_bitvec_xy},
};

/// 浮動小数点数のボクセル座標
/// DDAアルゴリズムで使用する
#[derive(Debug, Clone, Copy)]
pub(crate) struct VoxelFloat {
    pub(crate) z: u8,
    pub(crate) f: f64,
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl VoxelFloat {
    /// 整数ボクセル座標に変換
    pub(crate) fn to_voxel(&self) -> Voxel {
        Voxel {
            z: self.z,
            f: self.f.floor() as i64,
            x: self.x.floor() as u64,
            y: self.y.floor() as u64,
        }
    }
}

/// 整数のボクセル座標
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Voxel {
    pub(crate) z: u8,
    pub(crate) f: i64,
    pub(crate) x: u64,
    pub(crate) y: u64,
}

impl Voxel {
    /// EncodeIDに変換
    pub(crate) fn to_encode_id(&self) -> EncodeID {
        EncodeID {
            f: to_bitvec_f(self.z, self.f),
            x: to_bitvec_xy(self.z, self.x),
            y: to_bitvec_xy(self.z, self.y),
        }
    }

    /// 浮動小数点ボクセル座標に変換（中心座標）
    pub(crate) fn to_voxel_float(&self) -> VoxelFloat {
        VoxelFloat {
            z: self.z,
            f: self.f as f64 + 0.5,
            x: self.x as f64 + 0.5,
            y: self.y as f64 + 0.5,
        }
    }

    /// 座標を調整した新しいVoxelを返す
    pub(crate) fn adjust(&self, df: i64, dx: i64, dy: i64) -> Voxel {
        Voxel {
            z: self.z,
            f: self.f + df,
            x: (self.x as i64 + dx) as u64,
            y: (self.y as i64 + dy) as u64,
        }
    }

    /// 指定された軸方向のボクセルの長さ（メートル）を取得
    /// f: 高度方向、x: 経度方向、y: 緯度方向
    pub(crate) fn get_length(&self, axis: &str) -> f64 {
        match axis {
            "f" => {
                // 高度方向: 1ボクセル = 2^(25-z) メートル
                2_f64.powi(25 - self.z as i32)
            }
            "x" | "y" => {
                // x/y方向: 地球の円周 / 2^z
                // 赤道での近似値（約40075km / 2^z）
                let earth_circumference = 40_075_000.0; // メートル
                let n = 2_f64.powi(self.z as i32);
                earth_circumference / n
            }
            _ => 0.0,
        }
    }
}

/// 座標から浮動小数点ボクセル座標に変換
pub(crate) fn coordinate_to_voxel_float(coord: &crate::point::Coordinate, z: u8) -> VoxelFloat {
    let lat = coord.latitude;
    let lon = coord.longitude;
    let alt = coord.altitude;

    // 高度 h -> f (Python の h_to_f を Rust に移植)
    let factor = 2_f64.powi(z as i32 - 25);
    let f = factor * alt;

    // 経度 lon -> x
    let n = 2u64.pow(z as u32) as f64;
    let x = (lon + 180.0) / 360.0 * n;

    // 緯度 lat -> y (Web Mercator)
    let lat_rad = lat.to_radians();
    let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * n;

    VoxelFloat { z, f, x, y }
}

/// 3D DDAアルゴリズムで2点間のボクセルを取得
pub(crate) fn get_line_voxels_dda(vf1: VoxelFloat, vf2: VoxelFloat) -> Vec<Voxel> {
    let mut voxels = Vec::new();

    let z = vf1.z;
    let (ff1, xf1, yf1) = (vf1.f, vf1.x, vf1.y);
    let (ff2, xf2, yf2) = (vf2.f, vf2.x, vf2.y);

    let df = ff2 - ff1;
    let dx = xf2 - xf1;
    let dy = yf2 - yf1;

    let step_f: i64 = if df >= 0.0 { 1 } else { -1 };
    let step_x: i64 = if dx >= 0.0 { 1 } else { -1 };
    let step_y: i64 = if dy >= 0.0 { 1 } else { -1 };

    let adf = df.abs();
    let adx = dx.abs();
    let ady = dy.abs();

    // 始点
    let cur_voxel = vf1.to_voxel();
    voxels.push(cur_voxel);
    let (mut cur_f, mut cur_x, mut cur_y) = (cur_voxel.f, cur_voxel.x as i64, cur_voxel.y as i64);

    let v2 = vf2.to_voxel();
    let (f2, x2, y2) = (v2.f, v2.x as i64, v2.y as i64);

    // 1ボクセル進むのに必要な距離の比率
    let t_delta_f = if adf != 0.0 { 1.0 / adf } else { f64::INFINITY };
    let t_delta_x = if adx != 0.0 { 1.0 / adx } else { f64::INFINITY };
    let t_delta_y = if ady != 0.0 { 1.0 / ady } else { f64::INFINITY };

    // 次の境界までの距離の初期値
    let t_max_f = if step_f > 0 {
        (cur_f as f64 + 1.0 - ff1) * t_delta_f
    } else {
        (ff1 - cur_f as f64) * t_delta_f
    };

    let t_max_x = if step_x > 0 {
        (cur_x as f64 + 1.0 - xf1) * t_delta_x
    } else {
        (xf1 - cur_x as f64) * t_delta_x
    };

    let t_max_y = if step_y > 0 {
        (cur_y as f64 + 1.0 - yf1) * t_delta_y
    } else {
        (yf1 - cur_y as f64) * t_delta_y
    };

    let mut t_max_f = t_max_f;
    let mut t_max_x = t_max_x;
    let mut t_max_y = t_max_y;

    let max_i = adf + adx + ady + 3.0;
    let mut i = 0;

    while !(cur_x == x2 && cur_y == y2 && cur_f == f2) {
        // 最も近い境界を持つ軸を進める（これにより順序が保証される）
        if t_max_x < t_max_y {
            if t_max_x < t_max_f {
                cur_x += step_x;
                t_max_x += t_delta_x;
            } else {
                cur_f += step_f;
                t_max_f += t_delta_f;
            }
        } else if t_max_y < t_max_f {
            cur_y += step_y;
            t_max_y += t_delta_y;
        } else {
            cur_f += step_f;
            t_max_f += t_delta_f;
        }

        voxels.push(Voxel {
            z,
            f: cur_f,
            x: cur_x as u64,
            y: cur_y as u64,
        });

        if i as f64 > max_i {
            break;
        }

        i = i + 1;
    }

    voxels
}

/// 分割法でECEF空間を補間し、DDAアルゴリズムで線分上のボクセルを取得
fn get_voxels_along_line_divided<P: Point>(
    z: u8,
    point1: P,
    point2: P,
    divide_num: usize,
) -> Vec<Voxel> {
    let ecef1 = point1.to_ecef();
    let ecef2 = point2.to_ecef();

    let (x1, y1, z1) = (ecef1.x, ecef1.y, ecef1.z);
    let (x2, y2, z2) = (ecef2.x, ecef2.y, ecef2.z);

    // 分割点を計算
    let mut division_points = Vec::with_capacity(divide_num + 1);
    for i in 0..=divide_num {
        let t = i as f64 / divide_num as f64;
        let x = x1 * (1.0 - t) + x2 * t;
        let y = y1 * (1.0 - t) + y2 * t;
        let z_pos = z1 * (1.0 - t) + z2 * t;
        // ECEFをCoordinateに変換
        let coord = ECEF::new(x, y, z_pos).to_coordinate();
        division_points.push(coord);
    }

    // 始点のボクセル
    let mut voxels = vec![coordinate_to_voxel_float(&point1.to_coordinate(), z).to_voxel()];

    // 各分割区間でDDAを実行
    for j in 0..divide_num {
        let vf1 = coordinate_to_voxel_float(&division_points[j], z);
        let vf2 = coordinate_to_voxel_float(&division_points[j + 1], z);
        let mut line = get_line_voxels_dda(vf1, vf2);
        if !line.is_empty() {
            // 最初の点は前の区間と重複するので削除
            line.remove(0);
            voxels.extend(line);
        }
    }

    voxels
}

pub fn line<P: Point>(z: u8, a: P, b: P) -> EncodeIDSet {
    let ecef_a = a.to_ecef();
    let ecef_b = b.to_ecef();

    let coordinate_a = a.to_coordinate();
    let coordinate_b = b.to_coordinate();

    // --- ステップ計算 ---
    let dx = ecef_a.x - ecef_b.x;
    let dy = ecef_a.y - ecef_b.y;
    let dz = ecef_a.z - ecef_b.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    // 最小緯度のラジアン値
    let min_lat_rad = coordinate_a
        .latitude
        .abs()
        .min(coordinate_b.latitude.abs())
        .to_radians();
    let r = 6_378_137.0_f64;
    let d = PI * r * min_lat_rad.cos() * 2f64.powi(-3 - z as i32);

    let divide_num = (distance / d).ceil() as usize;
    let divide_num = divide_num.max(1);

    let voxels = get_voxels_along_line_divided(z, a, b, divide_num);

    let mut result = EncodeIDSet::new();
    for voxel in voxels {
        result.uncheck_insert(voxel.to_encode_id());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Coordinate;

    #[test]
    fn test_line_same_point() {
        // 同じ点間の線は1つのボクセルのみを返す
        let point = Coordinate {
            latitude: 35.6809,
            longitude: 139.7673,
            altitude: 0.0,
        };
        let result = line(10, point, point);
        assert!(result.iter().count() >= 1);
    }

    #[test]
    fn test_line_basic() {
        // 東京駅から新宿駅への線
        let tokyo = Coordinate {
            latitude: 35.6809,
            longitude: 139.7673,
            altitude: 0.0,
        };
        let shinjuku = Coordinate {
            latitude: 35.6896,
            longitude: 139.6999,
            altitude: 0.0,
        };
        let result = line(15, tokyo, shinjuku);
        // 線は複数のボクセルを通過するはず
        assert!(result.iter().count() >= 1);
    }

    #[test]
    fn test_get_line_voxels_dda_same_point() {
        let vf = VoxelFloat {
            z: 10,
            f: 0.5,
            x: 5.5,
            y: 3.5,
        };
        let voxels = get_line_voxels_dda(vf, vf);
        assert_eq!(voxels.len(), 1);
    }

    #[test]
    fn test_get_line_voxels_dda_horizontal() {
        let vf1 = VoxelFloat {
            z: 10,
            f: 0.0,
            x: 0.5,
            y: 0.5,
        };
        let vf2 = VoxelFloat {
            z: 10,
            f: 0.0,
            x: 3.5,
            y: 0.5,
        };
        let voxels = get_line_voxels_dda(vf1, vf2);
        // 水平線: x=0,1,2,3 の4つのボクセル
        assert_eq!(voxels.len(), 4);
        assert_eq!(voxels[0].x, 0);
        assert_eq!(voxels[1].x, 1);
        assert_eq!(voxels[2].x, 2);
        assert_eq!(voxels[3].x, 3);
    }

    #[test]
    fn test_get_line_voxels_dda_diagonal() {
        let vf1 = VoxelFloat {
            z: 10,
            f: 0.0,
            x: 0.5,
            y: 0.5,
        };
        let vf2 = VoxelFloat {
            z: 10,
            f: 0.0,
            x: 2.5,
            y: 2.5,
        };
        let voxels = get_line_voxels_dda(vf1, vf2);
        // 対角線: (0,0) -> (1,1) -> (2,2) またはその近傍の経路
        assert!(voxels.len() >= 3);
        assert_eq!(voxels[0].x, 0);
        assert_eq!(voxels[0].y, 0);
        assert_eq!(voxels.last().unwrap().x, 2);
        assert_eq!(voxels.last().unwrap().y, 2);
    }
}
