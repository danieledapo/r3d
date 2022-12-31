use crate::ray::Ray;
use crate::{Aabb, Vec3};

/// Check if a sphere defined by `center` and `radius` intersects a `Ray`. If so
/// return the parameter of the intersection point closest to `ray.origin`.
pub fn ray_intersection(center: Vec3, radius: f64, ray: &Ray) -> Option<f64> {
    let oc = ray.origin - center;

    let a = ray.dir.dot(ray.dir);
    let b = oc.dot(ray.dir);
    let c = oc.dot(oc) - radius.powi(2);

    let discr = b.powi(2) - a * c;

    if discr < 0.0 {
        return None;
    }

    let t0 = (-b - discr.sqrt()) / a;
    if t0 > 1e-9 {
        return Some(t0);
    }

    let t1 = (-b + discr.sqrt()) / a;
    if t1 > 1e-9 {
        return Some(t1);
    }

    None
}

/// Calculate the normal of point `p` among all the possible spheres centered at
/// `centered`. Since the normal is simply defined as the direction from
/// `center` to `p`, the radius is not taken into account.
pub fn normal(center: Vec3, p: Vec3) -> Vec3 {
    (p - center).normalized()
}

/// Calculate the bounding box of a sphere.
pub fn bounding_box(center: Vec3, radius: f64) -> Aabb {
    let mut aabb = Aabb::new(center - radius);
    aabb.expand(center + radius);

    aabb
}

#[cfg(test)]
mod tests {
    use crate::v3;

    use super::*;

    #[test]
    fn test_ray_intersection() {
        let c = Vec3::zero();
        let r = 1.0;

        assert_eq!(
            ray_intersection(c, r, &Ray::new(v3(0.0, 0.0, -2.0), v3(0, 0, 1))),
            Some(1.0)
        );

        assert_eq!(
            ray_intersection(c, r, &Ray::new(v3(0, 0, 2), v3(0, 0, 1))),
            None
        );

        assert_eq!(
            ray_intersection(c, r, &Ray::new(v3(1, 0, 2), v3(0.0, 0.0, -1.0))),
            Some(2.0)
        );

        assert_eq!(
            ray_intersection(c, r, &Ray::new(v3(-20.0, 0.0, 2.0), v3(0.0, 0.0, -1.0))),
            None
        );

        assert_eq!(
            ray_intersection(c, r, &Ray::new(c, v3(0.0, 0.0, -1.0))),
            Some(1.0)
        );
        assert_eq!(ray_intersection(c, r, &Ray::new(c, v3(0, 0, 1))), Some(1.0));
    }

    #[test]
    fn test_normal() {
        assert_eq!(normal(Vec3::zero(), v3(3, 0, 0)), v3(1, 0, 0));

        assert_eq!(normal(v3(2, 1, 0), v3(2, 0, 0)), v3(0.0, -1.0, 0.0));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = bounding_box(Vec3::zero(), 5.0);
        assert_eq!(bbox.min(), v3(-5.0, -5.0, -5.0));
        assert_eq!(bbox.max(), v3(5, 5, 5));

        let bbox = bounding_box(v3(1.0, -2.0, 3.0), 10.0);
        assert_eq!(bbox.min(), v3(-9.0, -12.0, -7.0));
        assert_eq!(bbox.max(), v3(11, 8, 13));
    }
}
