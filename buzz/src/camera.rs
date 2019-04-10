use std::f64::consts::PI;

use geo::ray::Ray;
use geo::vec3::Vec3;

/// A `Camera` is an object that allows to cast rays towards a 3D point in world
/// space that is calculated from a 2D point in screen space.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    position: Vec3,
    target: Vec3,

    u: Vec3,
    v: Vec3,
    w: Vec3,

    m: f64,
}

impl Camera {
    /// Create a `Camera` positioned at `position` pointed towards the given
    /// `target`. The `Camera` also needs a vertical field of view (`fovy`) in
    /// degrees to know how much space it can see. Lastly it needs a vector
    /// `vup` that represents the up axis to properly orient the camera.
    ///
    /// These paremeters define the [viewing frustrum][0] of the camera.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Viewing_frustum
    pub fn look_at(position: Vec3, target: Vec3, vup: Vec3, fovy: f64) -> Self {
        let w = (target - position).normalized();
        let u = w.cross(&vup).normalized();
        let v = u.cross(&w).normalized();

        let m = 1.0 / (fovy * PI / 360.0).tan();

        Camera {
            position,
            target,

            u,
            v,
            w,
            m,
        }
    }

    /// Create a `Ray` that starts from the `Camera`'s position to the 3D space
    /// with a direction that makes it pass through a given 2D point inside the
    /// viewport. `u` and `v` are offsets to `x` and `y` respectively and can be
    /// usefuly to slightly change the ray direction.
    pub fn cast_ray(
        &self,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        (u, v): (f64, f64),
    ) -> Ray {
        let x = f64::from(x);

        // invert y coordinate because in world space (0, 0) lies at the center
        // and the y axis grows upwards while in image space (0, 0) is at the
        // top left and y grows downwards.
        let y = f64::from(height - y);

        let width = f64::from(width);
        let height = f64::from(height);

        let aspect = width / height;
        let ndcx = (x + u - 0.5) / (width - 1.0) * 2.0 - 1.0;
        let ndcy = (y + v - 0.5) / (height - 1.0) * 2.0 - 1.0;

        let mut rd = Vec3::zero();
        rd += self.u * ndcx * aspect;
        rd += self.v * ndcy;
        rd += self.w * self.m;
        rd.normalize();

        Ray::new(self.position, rd)
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, Ray, Vec3};

    #[test]
    fn test_look_at() {
        let c = Camera::look_at(
            Vec3::zero(),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
            90.0,
        );

        assert_eq!(c.position, Vec3::zero());
        assert_eq!(c.target, Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(c.u, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(c.v, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(c.w, Vec3::new(0.0, 0.0, -1.0));
        assert!((c.m - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cast_ray() {
        let c = Camera::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            45.0,
        );

        assert_eq!(
            c.cast_ray((200, 100), (400, 200), (0.0, 0.0)),
            Ray::new(c.position, Vec3::new(0.0, 0.0, -1.0))
        );

        assert_eq!(
            c.cast_ray((0, 0), (400, 200), (0.1, 0.3)),
            Ray::new(
                c.position,
                Vec3::new(
                    -0.6080963736185712,
                    0.30587950310963447,
                    -0.7325684472930473
                )
            )
        );

        assert_eq!(
            c.cast_ray((300, 150), (400, 200), (0.7, 0.8)),
            Ray::new(
                c.position,
                Vec3::new(0.37907933452636, -0.18567591183873863, -0.9065447114720293)
            )
        );
    }
}
