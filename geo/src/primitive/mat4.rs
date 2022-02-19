use crate::Vec3;

/// Trait for geometric objects that can be transformed by a `Mat4` affine
/// matrix.
pub trait Transform {
    fn transform(&self, mat: &Mat4) -> Self;
}

/// A 3D [affine transformation][0] matrix in homogeneous coordinates.
///
/// [0]: https://en.wikipedia.org/wiki/Affine_transformation
#[derive(Debug, PartialEq, Clone)]
pub struct Mat4 {
    /// Raw coefficients in column-major order.
    pub data: [[f64; 4]; 4],
}

impl Mat4 {
    /// Create the identity transformation matrix.
    pub fn identity() -> Self {
        Mat4 {
            data: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create a translation matrix from the given translation.
    pub fn translate(v: Vec3) -> Self {
        Mat4 {
            data: [
                [1.0, 0.0, 0.0, v.x],
                [0.0, 1.0, 0.0, v.y],
                [0.0, 0.0, 1.0, v.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create a scale matrix from the given scale factors.
    pub fn scale(v: Vec3) -> Self {
        Mat4 {
            data: [
                [v.x, 0.0, 0.0, 0.0],
                [0.0, v.y, 0.0, 0.0],
                [0.0, 0.0, v.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create a rotation matrix from the given direction and angle.
    pub fn rotate(mut v: Vec3, a: f64) -> Self {
        v.normalize();
        let c = a.cos();
        let s = a.sin();
        let m = 1.0 - c;

        Mat4 {
            data: [
                [
                    m * v.x * v.x + c,
                    m * v.x * v.y + v.z * s,
                    m * v.z * v.x - v.y * s,
                    0.0,
                ],
                [
                    m * v.x * v.y - v.z * s,
                    m * v.y * v.y + c,
                    m * v.y * v.z + v.x * s,
                    0.0,
                ],
                [
                    m * v.z * v.x + v.y * s,
                    m * v.y * v.z - v.x * s,
                    m * v.z * v.z + c,
                    0.0,
                ],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Return the transpose of the matrix.
    #[allow(clippy::needless_range_loop)]
    pub fn transpose(&self) -> Self {
        let mut data = [[0.0; 4]; 4];

        for r in 0..4 {
            for c in 0..4 {
                data[r][c] = data[c][r];
            }
        }

        Mat4 { data }
    }

    /// Return the inverse of the matrix.
    #[rustfmt::skip]
    pub fn inverse(&self) -> Self {
        let m = &self.data;
        let d = self.determinant();

        let mut data = [[0.0;4];4];
        data[0][0] = (m[1][2]*m[2][3]*m[3][1] - m[1][3]*m[2][2]*m[3][1] + m[1][3]*m[2][1]*m[3][2] - m[1][1]*m[2][3]*m[3][2] - m[1][2]*m[2][1]*m[3][3] + m[1][1]*m[2][2]*m[3][3]) / d;
	    data[0][1] = (m[0][3]*m[2][2]*m[3][1] - m[0][2]*m[2][3]*m[3][1] - m[0][3]*m[2][1]*m[3][2] + m[0][1]*m[2][3]*m[3][2] + m[0][2]*m[2][1]*m[3][3] - m[0][1]*m[2][2]*m[3][3]) / d;
	    data[0][2] = (m[0][2]*m[1][3]*m[3][1] - m[0][3]*m[1][2]*m[3][1] + m[0][3]*m[1][1]*m[3][2] - m[0][1]*m[1][3]*m[3][2] - m[0][2]*m[1][1]*m[3][3] + m[0][1]*m[1][2]*m[3][3]) / d;
	    data[0][3] = (m[0][3]*m[1][2]*m[2][1] - m[0][2]*m[1][3]*m[2][1] - m[0][3]*m[1][1]*m[2][2] + m[0][1]*m[1][3]*m[2][2] + m[0][2]*m[1][1]*m[2][3] - m[0][1]*m[1][2]*m[2][3]) / d;
	    data[1][0] = (m[1][3]*m[2][2]*m[3][0] - m[1][2]*m[2][3]*m[3][0] - m[1][3]*m[2][0]*m[3][2] + m[1][0]*m[2][3]*m[3][2] + m[1][2]*m[2][0]*m[3][3] - m[1][0]*m[2][2]*m[3][3]) / d;
	    data[1][1] = (m[0][2]*m[2][3]*m[3][0] - m[0][3]*m[2][2]*m[3][0] + m[0][3]*m[2][0]*m[3][2] - m[0][0]*m[2][3]*m[3][2] - m[0][2]*m[2][0]*m[3][3] + m[0][0]*m[2][2]*m[3][3]) / d;
	    data[1][2] = (m[0][3]*m[1][2]*m[3][0] - m[0][2]*m[1][3]*m[3][0] - m[0][3]*m[1][0]*m[3][2] + m[0][0]*m[1][3]*m[3][2] + m[0][2]*m[1][0]*m[3][3] - m[0][0]*m[1][2]*m[3][3]) / d;
	    data[1][3] = (m[0][2]*m[1][3]*m[2][0] - m[0][3]*m[1][2]*m[2][0] + m[0][3]*m[1][0]*m[2][2] - m[0][0]*m[1][3]*m[2][2] - m[0][2]*m[1][0]*m[2][3] + m[0][0]*m[1][2]*m[2][3]) / d;
	    data[2][0] = (m[1][1]*m[2][3]*m[3][0] - m[1][3]*m[2][1]*m[3][0] + m[1][3]*m[2][0]*m[3][1] - m[1][0]*m[2][3]*m[3][1] - m[1][1]*m[2][0]*m[3][3] + m[1][0]*m[2][1]*m[3][3]) / d;
	    data[2][1] = (m[0][3]*m[2][1]*m[3][0] - m[0][1]*m[2][3]*m[3][0] - m[0][3]*m[2][0]*m[3][1] + m[0][0]*m[2][3]*m[3][1] + m[0][1]*m[2][0]*m[3][3] - m[0][0]*m[2][1]*m[3][3]) / d;
	    data[2][2] = (m[0][1]*m[1][3]*m[3][0] - m[0][3]*m[1][1]*m[3][0] + m[0][3]*m[1][0]*m[3][1] - m[0][0]*m[1][3]*m[3][1] - m[0][1]*m[1][0]*m[3][3] + m[0][0]*m[1][1]*m[3][3]) / d;
	    data[2][3] = (m[0][3]*m[1][1]*m[2][0] - m[0][1]*m[1][3]*m[2][0] - m[0][3]*m[1][0]*m[2][1] + m[0][0]*m[1][3]*m[2][1] + m[0][1]*m[1][0]*m[2][3] - m[0][0]*m[1][1]*m[2][3]) / d;
	    data[3][0] = (m[1][2]*m[2][1]*m[3][0] - m[1][1]*m[2][2]*m[3][0] - m[1][2]*m[2][0]*m[3][1] + m[1][0]*m[2][2]*m[3][1] + m[1][1]*m[2][0]*m[3][2] - m[1][0]*m[2][1]*m[3][2]) / d;
	    data[3][1] = (m[0][1]*m[2][2]*m[3][0] - m[0][2]*m[2][1]*m[3][0] + m[0][2]*m[2][0]*m[3][1] - m[0][0]*m[2][2]*m[3][1] - m[0][1]*m[2][0]*m[3][2] + m[0][0]*m[2][1]*m[3][2]) / d;
	    data[3][2] = (m[0][2]*m[1][1]*m[3][0] - m[0][1]*m[1][2]*m[3][0] - m[0][2]*m[1][0]*m[3][1] + m[0][0]*m[1][2]*m[3][1] + m[0][1]*m[1][0]*m[3][2] - m[0][0]*m[1][1]*m[3][2]) / d;
        data[3][3] = (m[0][1]*m[1][2]*m[2][0] - m[0][2]*m[1][1]*m[2][0] + m[0][2]*m[1][0]*m[2][1] - m[0][0]*m[1][2]*m[2][1] - m[0][1]*m[1][0]*m[2][2] + m[0][0]*m[1][1]*m[2][2]) / d;

        Mat4 {data}
    }

    /// Return the determinant of the matrix.
    #[rustfmt::skip]
    pub fn determinant(&self) -> f64 {
        let d = &self.data;

        d[0][0]*d[1][1]*d[2][2]*d[3][3] - d[0][0]*d[1][1]*d[2][3]*d[3][2] +
        d[0][0]*d[1][2]*d[2][3]*d[3][1] - d[0][0]*d[1][2]*d[2][1]*d[3][3] +
        d[0][0]*d[1][3]*d[2][1]*d[3][2] - d[0][0]*d[1][3]*d[2][2]*d[3][1] -
        d[0][1]*d[1][2]*d[2][3]*d[3][0] + d[0][1]*d[1][2]*d[2][0]*d[3][3] -
        d[0][1]*d[1][3]*d[2][0]*d[3][2] + d[0][1]*d[1][3]*d[2][2]*d[3][0] -
        d[0][1]*d[1][0]*d[2][2]*d[3][3] + d[0][1]*d[1][0]*d[2][3]*d[3][2] +
        d[0][2]*d[1][3]*d[2][0]*d[3][1] - d[0][2]*d[1][3]*d[2][1]*d[3][0] +
        d[0][2]*d[1][0]*d[2][1]*d[3][3] - d[0][2]*d[1][0]*d[2][3]*d[3][1] +
        d[0][2]*d[1][1]*d[2][3]*d[3][0] - d[0][2]*d[1][1]*d[2][0]*d[3][3] -
        d[0][3]*d[1][0]*d[2][1]*d[3][2] + d[0][3]*d[1][0]*d[2][2]*d[3][1] -
        d[0][3]*d[1][1]*d[2][2]*d[3][0] + d[0][3]*d[1][1]*d[2][0]*d[3][2] -
        d[0][3]*d[1][2]*d[2][0]*d[3][1] + d[0][3]*d[1][2]*d[2][1]*d[3][0]
    }

    /// Transform the given normalized `Vec3` to another normalized `Vec3`.
    pub fn transform_normal(&self, p: &Vec3) -> Vec3 {
        let dx = self.data[0][0] * p.x + self.data[0][1] * p.y + self.data[0][2] * p.z;
        let dy = self.data[1][0] * p.x + self.data[1][1] * p.y + self.data[1][2] * p.z;
        let dz = self.data[2][0] * p.x + self.data[2][1] * p.y + self.data[2][2] * p.z;

        Vec3::new(dx, dy, dz).normalized()
    }
}

impl Transform for Mat4 {
    /// Matrix composition.
    #[allow(clippy::needless_range_loop)]
    fn transform(&self, other: &Mat4) -> Self {
        let mut data = [[0.0; 4]; 4];

        for r in 0..4 {
            for c in 0..4 {
                data[r][c] = self.data[r][0] * other.data[0][c]
                    + self.data[r][1] * other.data[1][c]
                    + self.data[r][2] * other.data[2][c]
                    + self.data[r][3] * other.data[3][c];
            }
        }

        Mat4 { data }
    }
}

impl Transform for Vec3 {
    fn transform(&self, m: &Mat4) -> Self {
        Vec3::new(
            m.data[0][0] * self.x + m.data[0][1] * self.y + m.data[0][2] * self.z + m.data[0][3],
            m.data[1][0] * self.x + m.data[1][1] * self.y + m.data[1][2] * self.z + m.data[1][3],
            m.data[2][0] * self.x + m.data[2][1] * self.y + m.data[2][2] * self.z + m.data[2][3],
        )
    }
}
