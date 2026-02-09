use crate::Coordinate;

/// A 3D vector type.
#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    /// Creates a new Vec3.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Creates a Vec3 from a Coordinate.
    pub fn from_coord(coord: &Coordinate) -> Self {
        Self::new(
            coord.as_longitude(),
            coord.as_latitude(),
            coord.as_altitude(),
        )
    }

    /// Returns the x component of the vector.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y component of the vector.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the z component of the vector.
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Vector addition.
    #[allow(dead_code)]
    pub fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x(), self.y + other.y(), self.z + other.z())
    }

    /// Vector subtraction.
    pub fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x(), self.y - other.y(), self.z - other.z())
    }

    /// Scalar multiplication.
    pub fn mul(self, scalar: f64) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    /// Dot product.
    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x() + self.y * other.y() + self.z * other.z()
    }

    /// Cross product.
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z() - self.z * other.y(),
            self.z * other.x() - self.x * other.z(),
            self.x * other.y() - self.y * other.x(),
        )
    }

    /// Returns the squared length of the vector.
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the length of the vector.
    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// Normalizes the vector (makes length 1).
    /// Returns None if length is nearly 0.
    pub fn normalize(self) -> Option<Vec3> {
        let len = self.length();
        if len < 1.0e-15 {
            None
        } else {
            Some(self.mul(1.0 / len))
        }
    }

    /// Returns the distance to another vector.
    #[allow(dead_code)]
    pub fn distance(self, other: Vec3) -> f64 {
        self.sub(other).length()
    }

    /// Returns the squared distance to another vector.
    #[allow(dead_code)]
    pub fn distance_sq(self, other: Vec3) -> f64 {
        self.sub(other).length_sq()
    }
}
