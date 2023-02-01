use crate::{ray::Ray, Aabb};
use crate::{spatial_index::Shape, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    /// Create a new Triangle with the given vertices.
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    /// Calculate the area of a triangle. If it is made up by 3
    /// collinear points then the area is 0.
    pub fn area(&self) -> f64 {
        let e0 = self.b - self.a;
        let e1 = self.c - self.a;

        e0.cross(e1).norm() / 2.0
    }

    /// Calculate the normal of a triangle.
    pub fn normal(&self) -> Vec3 {
        let e0 = self.b - self.a;
        let e1 = self.c - self.a;

        e0.cross(e1).normalized()
    }

    /// Calculate the [centroid][0] of a triangle.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Centroid
    ///
    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    /// Compute the [barycentric coordinates][0] of a point `p` inside a triangle
    /// and return them in a `Vec3`. Return `None` if `p` lies outside the triangle.
    ///
    /// [0]: https://en.wikipedia.org/wiki/Barycentric_coordinate_system
    ///
    pub fn barycentric(&self, p: &Vec3) -> Option<Vec3> {
        let e0 = self.c - self.a;
        let e1 = self.b - self.a;

        let ep = *p - self.a;

        let dot00 = e0.dot(e0);
        let dot01 = e0.dot(e1);
        let dot11 = e1.dot(e1);

        let den = dot00 * dot11 - dot01 * dot01;

        // collinear or degenerate triangle
        if den == 0.0 {
            return None;
        }

        let dot12 = e1.dot(ep);
        let dot02 = e0.dot(ep);

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

impl Shape for Triangle {
    type Intersection = f64;

    /// Return the parameter t of the intersection between the ray and a triangle if
    /// any.
    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;

        let px = ray.dir.y * e2.z - ray.dir.z * e2.y;
        let py = ray.dir.z * e2.x - ray.dir.x * e2.z;
        let pz = ray.dir.x * e2.y - ray.dir.y * e2.x;

        let det = e1.x * px + e1.y * py + e1.z * pz;
        if det.abs() < 1e-9 {
            return None;
        }

        let inv = 1.0 / det;
        let t = ray.origin - self.a;
        let u = (t.x * px + t.y * py + t.z * pz) * inv;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qx = t.y * e1.z - t.z * e1.y;
        let qy = t.z * e1.x - t.x * e1.z;
        let qz = t.x * e1.y - t.y * e1.x;
        let v = (ray.dir.x * qx + ray.dir.y * qy + ray.dir.z * qz) * inv;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let d = (e2.x * qx + e2.y * qy + e2.z * qz) * inv;
        if d < 1e-9 {
            None
        } else {
            Some(d)
        }
    }

    /// Return the bounding box of a triangle.
    fn bbox(&self) -> Aabb {
        let mut aabb = Aabb::new(self.a);
        aabb.expand(self.b);
        aabb.expand(self.c);
        aabb
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
        let v0 = Vec3::new(-20.0, -20.0, 0.0);
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(-10.0, -2.0, 0.0);

        let tri = Triangle::new(v0, v1, v2);

        // triangle vertices always have valid bary coords
        assert_eq!(tri.barycentric(&v0), Some(Vec3::new(1.0, 0.0, 0.0)));
        assert_eq!(tri.barycentric(&v1), Some(Vec3::new(0.0, 1.0, 0.0)));
        assert_eq!(tri.barycentric(&v2), Some(Vec3::new(0.0, 0.0, 1.0)));

        // random point inside has valid coords
        assert_eq!(
            tri.barycentric(&(v0 * 0.25 + v1 * 0.25 + v2 * 0.5)),
            Some(Vec3::new(0.25, 0.25, 0.5))
        );

        // outside point has not valid coords
        assert_eq!(tri.barycentric(&Vec3::new(10.0, 0.0, 0.0)), None);
        assert_eq!(tri.barycentric(&Vec3::new(-5.0, -10.0, 0.0)), None);

        // collinear triangle doesn't return valid bary coords
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
