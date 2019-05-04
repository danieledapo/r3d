use crate::ray::Ray;
use crate::{Aabb, Vec3};

/// Return the infinite bounding box of an infinite plane.
pub fn bbox() -> Aabb {
    use std::f64::{INFINITY, NEG_INFINITY};

    Aabb::new(Vec3::new(NEG_INFINITY, NEG_INFINITY, NEG_INFINITY))
        .expanded(&Vec3::new(INFINITY, INFINITY, INFINITY))
}

/// Calculate the intersection between an infinite plane defined by a point and
/// a normal. The t parameter is returned if an intersection is found.
pub fn intersection(origin: Vec3, normal: Vec3, ray: &Ray) -> Option<f64> {
    let d = normal.dot(&ray.dir);
    if d.abs() < 1e-6 {
        return None;
    }

    let a = origin - ray.origin;
    let t = a.dot(&normal) / d;
    if t < 1e-6 {
        return None;
    }

    Some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersection() {
        assert_eq!(
            intersection(
                Vec3::zero(),
                Vec3::new(0.0, 0.0, 1.0),
                &Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0))
            ),
            Some(5.0)
        );

        assert_eq!(
            intersection(
                Vec3::zero(),
                Vec3::new(0.0, 0.0, 1.0),
                &Ray::new(Vec3::new(-5.0, 0.0, 5.0), Vec3::new(0.5, 0.0, -0.5))
            ),
            Some(10.0)
        );

        assert_eq!(
            intersection(
                Vec3::zero(),
                Vec3::new(0.0, 0.0, 1.0),
                &Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 1.0))
            ),
            None
        );
    }
}
