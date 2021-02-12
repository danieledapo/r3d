use std::f64::consts::PI;

use geo::{
    mat4::{Mat4, Transform},
    Vec3,
};

#[derive(Debug)]
pub struct Camera {
    eye: Vec3,
    camera_to_world: Mat4,
    matrix: Mat4,
}

impl Camera {
    pub fn look_at(position: Vec3, target: Vec3, vup: Vec3) -> Self {
        let f = (target - position).normalized();
        let s = f.cross(&vup).normalized();
        let u = s.cross(&f).normalized();

        let camera_to_world = Mat4 {
            data: [
                [s.x, u.x, -f.x, position.x],
                [s.y, u.y, -f.y, position.y],
                [s.z, u.z, -f.z, position.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        Self {
            eye: position,
            matrix: camera_to_world.clone(),
            camera_to_world,
        }
    }

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

        self.matrix = projection.transform(&self.camera_to_world.inverse());
        self
    }

    pub fn eye(&self) -> Vec3 {
        self.eye
    }

    pub fn project(&self, v: Vec3) -> Vec3 {
        let p = v.transform(&self.matrix);

        let m = &self.matrix.data;
        let w = m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3];
        p / w
    }
}
