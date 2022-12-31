use std::ops::Mul;

use crate::{mat4::Mat4, Axis, Vec3};

/// A `Ray` is a line starting from a given point and going towards a given
/// direction.
#[derive(Debug, PartialEq, Clone)]
pub struct Ray {
    /// The origin of the `Ray`.
    pub origin: Vec3,

    /// The direction, possibly not normalized, of the `Ray`.
    pub dir: Vec3,
}

impl Ray {
    /// Create a new `Ray` with the given origin and direction. The direction
    /// doesn't have to be normalized.
    pub fn new(origin: Vec3, dir: Vec3) -> Self {
        Ray { origin, dir }
    }

    /// Get the point on a `Ray` at the given parameter `t`.
    pub fn point_at(&self, t: f64) -> Vec3 {
        self.origin + self.dir * t
    }

    /// Get the parameter `t` of the given point `p`. If `p` does not lie on the
    /// `Ray` then `None` is returned. This function can return a negative value
    /// if point is on opposite ray.
    pub fn t_of(&self, p: Vec3) -> Option<f64> {
        // if p lies on self then `d / self.dir = t` where t must be the same
        // (or INFINITY, -INFINITY, Nan) among all the coordinates.
        let d = p - self.origin;

        let mut pv = None;

        for axis in &[Axis::X, Axis::Y, Axis::Z] {
            let t = d[*axis] / self.dir[*axis];

            // if t is not finite then self.dir[*axis] must be 0.0 in which case
            // the coordinate is fixed and we can just check that it matches
            // between `p` and `self.origin`
            if !t.is_finite() {
                if p[*axis] != self.origin[*axis] {
                    return None;
                }

                continue;
            }

            match pv {
                None => pv = Some(t),
                Some(pt) if pt != t => return None,
                Some(_pt) => {}
            }
        }

        pv
    }

    /// Reflect this `Ray` and return the reflected direction.
    pub fn reflect(&self) -> Vec3 {
        self.origin - self.dir * self.origin.dot(self.dir) * 2.0
    }

    /// Try to refract this ray using [Snell's law][0] given a refractive index.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Snell%27s_law
    pub fn refract(&self, refraction_index: f64) -> Option<Vec3> {
        let uv = self.origin.normalized();
        let dt = uv.dot(self.dir);
        let discriminant = 1.0 - refraction_index.powi(2) * (1.0 - dt.powi(2));

        if discriminant >= 0.0 {
            Some((uv - self.dir * dt) * refraction_index - self.dir * discriminant.sqrt())
        } else {
            None
        }
    }
}

impl Mul<&Mat4> for Ray {
    type Output = Ray;

    fn mul(self, mat: &Mat4) -> Self::Output {
        Ray::new(self.origin * mat, mat.transform_normal(&self.dir))
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

    #[test]
    fn test_refract() {
        assert_eq!(
            Ray::new(Vec3::new(5.0, 1.0, 3.0), Vec3::new(0.0, 1.0, 0.0)).refract(1.5),
            None
        );

        assert_eq!(
            Ray::new(Vec3::new(3.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)).refract(1.5),
            Some(Vec3::new(-0.8803408430829504, 0.0, 0.4743416490252569))
        );
    }

    #[test]
    fn test_t_of() {
        let r = Ray::new(Vec3::new(1.0, -1.0, 0.0), Vec3::new(2.0, 1.0, 5.0));

        assert_eq!(r.t_of(Vec3::new(1.0, -1.0, 0.0)), Some(0.0));
        assert_eq!(r.t_of(Vec3::new(3.0, 0.0, 5.0)), Some(1.0));
        assert_eq!(r.t_of(Vec3::new(0.0, -1.5, -2.5)), Some(-0.5));
        assert_eq!(r.t_of(Vec3::new(10.0, -1.5, -2.5)), None);

        assert_eq!(
            Ray::new(Vec3::new(1.0, -1.0, 3.0), Vec3::new(0.0, 1.0, 0.0))
                .t_of(Vec3::new(1.0, 0.0, 3.0)),
            Some(1.0)
        );
        assert_eq!(
            Ray::new(Vec3::new(1.0, -1.0, 3.0), Vec3::new(0.0, 1.0, 0.0))
                .t_of(Vec3::new(8.0, 0.0, 3.0)),
            None
        );
    }
}
