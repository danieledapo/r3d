#![allow(clippy::many_single_char_names)]

use std::f64::consts::PI;

use rand::Rng;

use geo::{ray::Ray, Vec3};

/// A `Camera` is an object that allows to cast rays towards a 3D point in world
/// space that is calculated from a 2D point in screen space.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    position: Vec3,
    target: Vec3,

    // x,y,z unit vectors
    u: Vec3,
    v: Vec3,
    w: Vec3,

    // fovy factor
    m: f64,

    lens: Option<Lens>,
}

#[derive(Debug, Clone, PartialEq)]
struct Lens {
    aperture_radius: f64,
    focal_distance: f64,
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
        let u = w.cross(vup).normalized();
        let v = u.cross(w).normalized();

        let m = 1.0 / (fovy * PI / 360.0).tan();

        Camera {
            position,
            target,

            u,
            v,
            w,
            m,

            lens: None,
        }
    }

    /// Change the camera focal point and aperture radius to change the depth of
    /// view of the scene.
    pub fn with_focus(mut self, focal_point: Vec3, aperture_radius: f64) -> Camera {
        self.lens = Some(Lens {
            focal_distance: (focal_point - self.position).norm(),
            aperture_radius,
        });

        self
    }

    /// Create a `Ray` that starts from the `Camera`'s position to the 3D space
    /// with a direction that makes it pass through a given 2D point inside the
    /// viewport. A `Rng` is needed to slightly perturb the generated rays to
    /// improve the quality of the rendering.
    pub fn cast_ray(
        &self,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        rng: &mut impl Rng,
    ) -> Ray {
        let x = f64::from(x);

        // invert y coordinate because in world space (0, 0) lies at the center
        // and the y axis grows upwards while in image space (0, 0) is at the
        // top left and y grows downwards.
        let y = f64::from(height - y);

        let width = f64::from(width);
        let height = f64::from(height);

        let u: f64 = rng.gen();
        let v: f64 = rng.gen();

        let aspect = width / height;
        let ndcx = (x + u - 0.5) / (width - 1.0) * 2.0 - 1.0;
        let ndcy = (y + v - 0.5) / (height - 1.0) * 2.0 - 1.0;

        let mut rd = Vec3::zero();
        rd += self.u * ndcx * aspect;
        rd += self.v * ndcy;
        rd += self.w * self.m;
        rd.normalize();

        match self.lens {
            Some(Lens {
                aperture_radius,
                focal_distance,
            }) => {
                let focal_point = self.position + rd * focal_distance;
                let angle = rng.gen::<f64>() * 2.0 * PI;
                let radius = rng.gen::<f64>() * aperture_radius;

                let p = self.position
                    + self.u * (angle.cos() * radius)
                    + self.v * (angle.sin() * radius);

                Ray::new(p, (focal_point - p).normalized())
            }
            None => Ray::new(self.position, rd),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

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
        let mut rng = XorShiftRng::seed_from_u64(0);

        let c = Camera::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            45.0,
        );

        assert_eq!(
            c.cast_ray((200, 100), (400, 200), &mut rng),
            Ray::new(
                c.position,
                Vec3::new(
                    0.0036926676998767344,
                    0.0018201043881227754,
                    -0.9999915256767302
                )
            )
        );

        assert_eq!(
            c.cast_ray((0, 0), (400, 200), &mut rng),
            Ray::new(
                c.position,
                Vec3::new(
                    -0.6080162292347211,
                    0.30680626952501405,
                    -0.7322473475317856
                )
            )
        );
    }

    #[test]
    fn test_cast_ray_with_focus() {
        let mut rng = XorShiftRng::seed_from_u64(0);

        let c = Camera::look_at(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            45.0,
        )
        .with_focus(Vec3::new(0.0, 0.0, 0.0), 1.0);

        assert_eq!(
            c.cast_ray((300, 150), (400, 200), &mut rng),
            Ray::new(
                Vec3::new(0.6289473439268372, 0.15601649433924777, 5.0),
                Vec3::new(
                    0.26276272257151023,
                    -0.22585126579607048,
                    -0.9380548797192626
                )
            )
        );
    }
}
