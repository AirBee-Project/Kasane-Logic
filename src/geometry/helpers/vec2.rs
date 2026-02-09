/// A 2D vector type.
#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    /// Creates a new Vec2.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Returns the x component.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y component.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Vector addition.
    #[allow(dead_code)]
    pub fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x(), self.y + other.y())
    }

    /// Vector subtraction.
    #[allow(dead_code)]
    pub fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x(), self.y - other.y())
    }

    /// Scalar multiplication.
    #[allow(dead_code)]
    pub fn mul(self, scalar: f64) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }

    /// Dot product.
    #[allow(dead_code)]
    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x() + self.y * other.y()
    }

    /// Returns the squared length of the vector.
    #[allow(dead_code)]
    pub fn length_sq(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Returns the length of the vector.
    #[allow(dead_code)]
    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    /// Normalizes the vector (makes length 1).
    /// Returns None if length is nearly 0.
    #[allow(dead_code)]
    pub fn normalize(self) -> Option<Vec2> {
        let len = self.length();
        if len < 1.0e-15 {
            None
        } else {
            Some(self.mul(1.0 / len))
        }
    }

    /// Returns the distance to another vector.
    #[allow(dead_code)]
    pub fn distance(self, other: Vec2) -> f64 {
        self.sub(other).length()
    }

    /// Returns the squared distance to another vector.
    #[allow(dead_code)]
    pub fn distance_sq(self, other: Vec2) -> f64 {
        self.sub(other).length_sq()
    }
}
