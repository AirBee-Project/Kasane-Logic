use crate::spatial_id::SpatialId;
use crate::{
    geometry::{
        coordinate::Coordinate,
        constants::{WGS84_A, WGS84_B},
    },
    spatial_id::single::SingleId,
};

const EPS: f64 = 1e-12;

/// 補助球（unit sphere）上の3次元ベクトル
#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    /// WGS-84 楕円体上の座標 → 補助球（縮緯度）
    fn from_wgs84(c: Coordinate) -> Self {
        let lat = c.as_latitude().to_radians();
        let lon = c.as_longitude().to_radians();

        // 縮緯度（parametric latitude）
        let beta = ((WGS84_B / WGS84_A) * lat.tan()).atan();
        let cb = beta.cos();

        Self {
            x: cb * lon.cos(),
            y: cb * lon.sin(),
            z: beta.sin(),
        }
    }

    fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    fn cross(self, o: Self) -> Self {
        Self {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }
}

/// 補助球上の球面三角形内部判定
fn spherical_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> bool {
    let ab = a.cross(b);
    let bc = b.cross(c);
    let ca = c.cross(a);

    let s1 = ab.dot(p);
    let s2 = bc.dot(p);
    let s3 = ca.dot(p);

    (s1 >= -EPS && s2 >= -EPS && s3 >= -EPS)
        || (s1 <= EPS && s2 <= EPS && s3 <= EPS)
}

/// 球面四角形（タイル）内部判定
fn inside_spherical_quad(p: Vec3, q: [Vec3; 4]) -> bool {
    spherical_triangle(p, q[0], q[1], q[2])
        || spherical_triangle(p, q[0], q[2], q[3])
}

/// 補助球上の測地線（大円弧）交差判定
fn great_arc_intersect(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> bool {
    let n1 = a.cross(b);
    let n2 = c.cross(d);

    let s1 = n1.dot(c);
    let s2 = n1.dot(d);
    let s3 = n2.dot(a);
    let s4 = n2.dot(b);

    s1 * s2 < -EPS && s3 * s4 < -EPS
}

///
/// 楕円体（WGS-84）上の三角形と SingleId タイルの交差判定
/// （地表のみ・測地線ベース）
///
pub fn ellipsoid_triangle_intersects_single_id(
    tri: [Coordinate; 3],
    tile: &SingleId,
) -> bool {
    // --- 三角形（補助球） ---
    let a = Vec3::from_wgs84(tri[0]);
    let b = Vec3::from_wgs84(tri[1]);
    let c = Vec3::from_wgs84(tri[2]);
    let tri_v = [a, b, c];

    // --- タイル頂点（補助球） ---
    let verts = tile.vertices();
    let tile_v = [
        Vec3::from_wgs84(verts[0]),
        Vec3::from_wgs84(verts[1]),
        Vec3::from_wgs84(verts[3]),
        Vec3::from_wgs84(verts[2]),
    ];

    // ① 三角形頂点 ∈ タイル
    for &p in &tri_v {
        if inside_spherical_quad(p, tile_v) {
            return true;
        }
    }

    // ② タイル頂点 ∈ 三角形
    for &v in &tile_v {
        if spherical_triangle(v, a, b, c) {
            return true;
        }
    }

    // ③ 辺同士の交差
    let tri_edges = [(0, 1), (1, 2), (2, 0)];
    let tile_edges = [(0, 1), (1, 2), (2, 3), (3, 0)];

    for (i, j) in tri_edges {
        for (k, l) in tile_edges {
            if great_arc_intersect(tri_v[i], tri_v[j], tile_v[k], tile_v[l]) {
                return true;
            }
        }
    }

    false
}

