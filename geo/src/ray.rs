use crate::vec3::Vec3;

/// A `Ray` is a line starting from a given point and going towards a given
/// direction.
#[derive(Debug, PartialEq, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

impl Ray {
    /// Create a new `Ray` with the given origin and direction.
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin, dir }
    }

    /// Get the point on a `Ray` at the given parameter `t`.
    pub fn point_at(&self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }

    /// Reflect this `Ray` and return the reflected direction.
    pub fn reflect(&self) -> Vec3 {
        self.origin - self.dir * self.origin.dot(&self.dir) * 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Ray, Vec3};

    #[test]
    fn test_point_at() {
        let ray = Ray::new(Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(ray.point_at(0.0), ray.origin);
        assert_eq!(ray.point_at(1.0), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(ray.point_at(0.5), Vec3::new(0.0, 0.5, 0.0));
    }

    #[test]
    fn test_reflect() {
        assert_eq!(
            Ray::new(Vec3::new(5.0, 1.0, 3.0), Vec3::new(0.0, 1.0, 0.0)).reflect(),
            Vec3::new(5.0, -1.0, 3.0)
        );

        assert_eq!(
            Ray::new(Vec3::zero(), Vec3::new(0.0, 1.0, 0.0)).reflect(),
            Vec3::zero()
        );
    }
}
