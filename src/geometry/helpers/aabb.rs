use crate::geometry::helpers::vec3::Vec3;

/// 軸に沿った境界ボックス（Axis-Aligned Bounding Box）
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    /// 新しいAABBを作成
    ///
    /// # 引数
    /// * `min` - ボックスの最小コーナー
    /// * `max` - ボックスの最大コーナー
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// 三角形からAABBを作成
    ///
    /// # 引数
    /// * `v0`, `v1`, `v2` - 三角形の頂点
    pub fn from_triangle(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Self {
            min: Vec3::new(
                v0.x().min(v1.x()).min(v2.x()),
                v0.y().min(v1.y()).min(v2.y()),
                v0.z().min(v1.z()).min(v2.z()),
            ),
            max: Vec3::new(
                v0.x().max(v1.x()).max(v2.x()),
                v0.y().max(v1.y()).max(v2.y()),
                v0.z().max(v1.z()).max(v2.z()),
            ),
        }
    }

    /// 他のAABBと交差するか判定
    ///
    /// # 引数
    /// * `other` - 比較するAABB
    /// * `margin` - 判定に用いる余白
    pub fn intersects(&self, other: &AABB, margin: f64) -> bool {
        self.min.x() - margin <= other.max.x()
            && self.max.x() + margin >= other.min.x()
            && self.min.y() - margin <= other.max.y()
            && self.max.y() + margin >= other.min.y()
            && self.min.z() - margin <= other.max.z()
            && self.max.z() + margin >= other.min.z()
    }
}
