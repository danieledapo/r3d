use crate::vec3::Vec3;

/// A simple `Triangle`.
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub v0: Vec3,
    pub v1: Vec3,
    pub v2: Vec3,
}

impl Triangle {
    /// Create a new `Triangle`.
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Triangle { v1, v2, v0 }
    }

    /// Calculate the area of this `Triangle`. If the `Triangle` is made up by 3
    /// collinear points then the area is 0.
    pub fn area(&self) -> f64 {
        let e0 = self.v1 - self.v0;
        let e1 = self.v2 - self.v0;

        e0.cross(&e1).norm() / 2.0
    }

    /// Calculate the normal of this `Triangle`.
    pub fn normal(&self) -> Vec3 {
        let e0 = self.v1 - self.v0;
        let e1 = self.v2 - self.v0;

        e0.cross(&e1).normalized()
    }

    /// Calculate the [centroid][0] of this `Triangle`.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Centroid
    pub fn centroid(&self) -> Vec3 {
        (self.v0 + self.v1 + self.v2) / 3.0
    }

    /// Compute the [barycentric coordinates][0] of a point `p` inside this
    /// `Triangle` and return them in a `Vec3`. Return `None` if `p` lies
    /// outside this triangle.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Barycentric_coordinate_system
    pub fn barycentric(&self, p: &Vec3) -> Option<Vec3> {
        let e0 = self.v2 - self.v0;
        let e1 = self.v1 - self.v0;

        let ep = *p - self.v0;

        let dot00 = e0.dot(&e0);
        let dot01 = e0.dot(&e1);
        let dot11 = e1.dot(&e1);

        let den = dot00 * dot11 - dot01 * dot01;

        // collinear or degenerate triangle
        if den == 0.0 {
            return None;
        }

        let dot12 = e1.dot(&ep);
        let dot02 = e0.dot(&ep);

        let u = (dot11 * dot02 - dot01 * dot12) / den;
        let v = (dot00 * dot12 - dot01 * dot02) / den;

        // valid barycentric coordinates must always sum to 1 and each component
        // should be in [0, 1], if they do not then`p` is outside the triangle
        if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
            None
        } else {
            Some(Vec3::new(1.0 - u - v, v, u))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area() {
        assert_eq!(
            Triangle::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(50.0, 0.0, 0.0),
                Vec3::new(25.0, 10.0, 0.0)
            )
            .area(),
            250.0
        );

        assert_eq!(
            Triangle::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(50.0, 0.0, 0.0),
                Vec3::new(100.0, 0.0, 0.0)
            )
            .area(),
            0.0
        );
    }

    #[test]
    fn test_triangle_normal() {
        assert_eq!(
            Triangle::new(
                Vec3::new(2.0, 2.0, 2.0),
                Vec3::new(10.0, 15.0, 2.0),
                Vec3::new(4.0, 10.0, 2.0)
            )
            .normal(),
            Vec3::new(0.0, 0.0, 1.0)
        );

        assert_eq!(
            Triangle::new(
                Vec3::new(2.0, 2.0, 2.0),
                Vec3::new(10.0, 15.0, 2.0),
                Vec3::new(4.0, 10.0, 2.0)
            )
            .normal(),
            Vec3::new(0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn test_triangle_centroid() {
        assert_eq!(
            Triangle::new(
                Vec3::new(2.0, 5.0, 2.0),
                Vec3::new(10.0, 15.0, 2.0),
                Vec3::new(6.0, 10.0, 2.0)
            )
            .centroid(),
            Vec3::new(6.0, 10.0, 2.0)
        );
    }

    #[test]
    fn test_triangle_barycentric() {
        let tri = Triangle::new(
            Vec3::new(-20.0, -20.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(-10.0, -2.0, 0.0),
        );

        // triangle vertices
        assert_eq!(
            tri.barycentric(&Vec3::new(-20.0, -20.0, 0.0)),
            Some(Vec3::new(1.0, 0.0, 0.0))
        );

        assert_eq!(
            tri.barycentric(&Vec3::new(0.0, 0.0, 0.0)),
            Some(Vec3::new(0.0, 1.0, 0.0))
        );

        assert_eq!(
            tri.barycentric(&Vec3::new(-10.0, -2.0, 0.0)),
            Some(Vec3::new(0.0, 0.0, 1.0))
        );

        // random point inside
        assert_eq!(
            tri.barycentric(&(tri.v0 * 0.25 + tri.v1 * 0.25 + tri.v2 * 0.5)),
            Some(Vec3::new(0.25, 0.25, 0.5))
        );

        // outside
        assert_eq!(tri.barycentric(&Vec3::new(10.0, 0.0, 0.0)), None);
        assert_eq!(tri.barycentric(&Vec3::new(-5.0, -10.0, 0.0)), None);

        // collinear triangle
        assert_eq!(
            Triangle::new(
                Vec3::new(10.0, 10.0, 10.0),
                Vec3::new(20.0, 20.0, 20.0),
                Vec3::new(0.0, 0.0, 0.0)
            )
            .barycentric(&Vec3::new(10.0, 10.0, 10.0)),
            None
        );
    }
}
