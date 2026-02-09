use crate::geometry::helpers::vec3::Vec3;

/// An axis-aligned bounding box.
/// Used for collision detection.
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    /// Creates a new AABB.
    ///
    /// # Arguments
    /// * `min` - Minimum corner of the box
    /// * `max` - Maximum corner of the box
    #[allow(dead_code)]
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates an AABB from a triangle.
    ///
    /// # Arguments
    /// * `v0`, `v1`, `v2` - Triangle vertices
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

    /// Tests if this AABB intersects with another AABB.
    ///
    /// # Arguments
    /// * `other` - AABB to compare with
    /// * `margin` - Margin for the test
    pub fn intersects(&self, other: &AABB, margin: f64) -> bool {
        self.min.x() - margin <= other.max.x()
            && self.max.x() + margin >= other.min.x()
            && self.min.y() - margin <= other.max.y()
            && self.max.y() + margin >= other.min.y()
            && self.min.z() - margin <= other.max.z()
            && self.max.z() + margin >= other.min.z()
    }
}
