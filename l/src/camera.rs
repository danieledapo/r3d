use std::f64::consts::PI;

use geo::{mat4::Mat4, Vec3};

/// A `Camera` is an object that allows to cast rays towards a 3D point in world
/// space that is calculated from a 2D point in screen space.
#[derive(Debug)]
pub struct Camera {
    position: Vec3,
    camera_to_world: Mat4,
    matrix: Mat4,
}

impl Camera {
    /// Create a `Camera` positioned at `position` pointed towards the given
    /// `target`. Lastly it needs a vector `vup` that represents the up axis to
    /// properly orient the camera.
    ///
    /// These parameters define the [viewing frustrum][0] of the camera.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Viewing_frustum
    pub fn look_at(position: Vec3, target: Vec3, vup: Vec3) -> Self {
        let f = (target - position).normalized();
        let s = f.cross(vup).normalized();
        let u = s.cross(f).normalized();

        let camera_to_world = Mat4 {
            data: [
                [s.x, u.x, -f.x, position.x],
                [s.y, u.y, -f.y, position.y],
                [s.z, u.z, -f.z, position.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        Self {
            position,
            matrix: camera_to_world.clone(),
            camera_to_world,
        }
    }

    /// Set the `Camera` to use [Perspective projection][0] when projecting 3D
    /// points to 2D.
    ///
    /// To create the perspective projection matrix the vertical field of view
    /// is required along with the desired aspect ratio of the projection.
    /// Moreover, the values for the near and far plane are required and not in
    /// between of these planes are not projected.
    ///
    /// [0]: https://en.wikipedia.org/wiki/3D_projection#Perspective_projection
    #[rustfmt::skip]
    pub fn with_perspective_projection(
        mut self,
        fovy: f64,
        aspect: f64,
        near: f64,
        far: f64,
    ) -> Self {
        let ymax = near * (fovy * PI / 360.0).tan();
        let xmax = ymax * aspect;

        let t1 = 2.0 * near;
        let t2 = 2.0 * xmax;
        let t3 = 2.0 * ymax;
        let t4 = far - near;

        let projection = Mat4 {
            data: [
                [t1 / t2, 0.0,     0.0,                0.0],
                [0.0,     t1 / t3, 0.0,                0.0],
                [0.0,     0.0,     (-far - near) / t4, -t1 * far / t4],
                [0.0,     0.0,     -1.0,               0.0],
            ],
        };

        self.matrix = projection * &self.camera_to_world.inverse();
        self
    }

    /// Return the position where the camera is located.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Project the given point in 3D space to 2D as seen by this `Camera`.
    pub fn project(&self, v: Vec3) -> Vec3 {
        let p = v * &self.matrix;

        let m = &self.matrix.data;
        let w = m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3];
        p / w
    }
}
