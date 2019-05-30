//! Simple [Constructive Solid Geometry][0] framework.
//!
//! [0]: https://en.wikipedia.org/wiki/Constructive_solid_geometry
//!
//! The distance functions are based on
//! http://www.iquilezles.org/www/articles/distfunctions/distfunctions.htm
//!

use geo::mat4::{Mat4, Transform};
use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

use crate::{Hit, Surface};

#[derive(Debug)]
pub struct SdfGeometry<S> {
    sdf: S,
}

impl<S: SignedDistanceFunction> SdfGeometry<S> {
    pub fn new(sdf: S) -> Self {
        SdfGeometry { sdf }
    }
}

pub trait SignedDistanceFunction: Sized + std::fmt::Debug {
    fn dist(&self, p: &Vec3) -> f64;
    fn bbox(&self) -> Aabb;

    fn transformed(self, xform: Mat4) -> Transformed<Self> {
        let inverse_matrix = xform.inverse();
        Transformed {
            sdf: self,
            matrix: xform,
            inverse_matrix,
        }
    }

    fn union<S: SignedDistanceFunction>(self, other: S) -> Union<Self, S> {
        Union {
            left: self,
            right: other,
        }
    }

    fn intersection<S: SignedDistanceFunction>(self, other: S) -> Intersection<Self, S> {
        Intersection {
            left: self,
            right: other,
        }
    }

    fn difference<S: SignedDistanceFunction>(self, other: S) -> Difference<Self, S> {
        Difference {
            left: self,
            right: other,
        }
    }
}

impl<S: SignedDistanceFunction> Surface for SdfGeometry<S> {
    fn normal_at(&self, p: Vec3) -> Vec3 {
        let e = 0.000001;
        let Vec3 { x, y, z } = p;
        let s = &self.sdf;
        let n = Vec3::new(
            s.dist(&Vec3::new(x + e, y, z)) - s.dist(&Vec3::new(x - e, y, z)),
            s.dist(&Vec3::new(x, y + e, z)) - s.dist(&Vec3::new(x, y - e, z)),
            s.dist(&Vec3::new(x, y, z + e)) - s.dist(&Vec3::new(x, y, z - e)),
        );
        n.normalized()
    }
}

impl<'s, S: SignedDistanceFunction> Shape<'s> for SdfGeometry<S> {
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        self.sdf.bbox()
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let epsilon = 0.00001;
        let jump_size = 0.001;

        let (t1, t2) = box_intersect(&self.bbox(), ray);
        if t2 < t1 || t2 < 0.0 {
            return None;
        }

        let mut t = t1.max(0.0001);
        let mut jump = true;

        // ray marching
        for _ in 0..1000 {
            let mut d = self.sdf.dist(&ray.point_at(t));

            if jump && d < 0.0 {
                t -= jump_size;
                jump = false;
                continue;
            }

            if d < epsilon {
                return Some(Hit {
                    surface: self,
                    t,
                    point_and_normal: None,
                });
            }

            if jump && d < jump_size {
                d = jump_size;
            }

            t += d;

            if t > t2 {
                break;
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    radius: f64,
}

impl Sphere {
    pub fn new(r: f64) -> Self {
        Sphere { radius: r }
    }
}

impl SignedDistanceFunction for Sphere {
    fn bbox(&self) -> Aabb {
        geo::sphere::bounding_box(Vec3::zero(), self.radius)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        p.norm() - self.radius
    }
}

#[derive(Debug, Clone)]
pub struct Cube {
    size: Vec3,
}

impl Cube {
    pub fn new(size: Vec3) -> Self {
        Cube { size }
    }
}

impl SignedDistanceFunction for Cube {
    fn bbox(&self) -> Aabb {
        let d = self.size / 2.0;
        Aabb::new(-d).expanded(&d)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let mut x = if p.x < 0.0 { -p.x } else { p.x };
        let mut y = if p.y < 0.0 { -p.y } else { p.y };
        let mut z = if p.z < 0.0 { -p.z } else { p.z };

        x -= self.size.x / 2.0;
        y -= self.size.y / 2.0;
        z -= self.size.z / 2.0;

        let a = x.max(y).max(z).min(0.0);

        x = x.max(0.0);
        y = y.max(0.0);
        z = z.max(0.0);

        let b = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
        a + b
    }
}

#[derive(Debug, Clone)]
pub struct Cylinder {
    radius: f64,
    height: f64,
}

impl Cylinder {
    pub fn new(radius: f64, height: f64) -> Self {
        Cylinder { radius, height }
    }
}

impl SignedDistanceFunction for Cylinder {
    fn bbox(&self) -> Aabb {
        Aabb::new(Vec3::new(-self.radius, -self.height / 2.0, -self.radius)).expanded(&Vec3::new(
            self.radius,
            self.height / 2.0,
            self.radius,
        ))
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let mut x = (p.x.powi(2) + p.z.powi(2)).sqrt();
        if x < 0.0 {
            x = -x;
        }

        let mut y = if p.y < 0.0 { -p.y } else { p.y };

        x -= self.radius;
        y -= self.height / 2.0;
        let a = x.max(y).min(0.0);

        x = x.max(0.0);
        y = y.max(0.0);
        let b = (x.powi(2) + y.powi(2)).sqrt();

        a + b
    }
}

#[derive(Debug)]
pub struct Transformed<S> {
    sdf: S,
    matrix: Mat4,
    inverse_matrix: Mat4,
}

impl<S: SignedDistanceFunction> SignedDistanceFunction for Transformed<S> {
    fn bbox(&self) -> Aabb {
        self.sdf.bbox().transform(&self.matrix)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let q = p.transform(&self.inverse_matrix);
        self.sdf.dist(&q)
    }
}

#[derive(Debug)]
pub struct Union<S1, S2> {
    left: S1,
    right: S2,
}

impl<S1, S2> SignedDistanceFunction for Union<S1, S2>
where
    S1: SignedDistanceFunction,
    S2: SignedDistanceFunction,
{
    fn bbox(&self) -> Aabb {
        let mut b = self.left.bbox();
        b.union(&self.right.bbox());
        b
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.min(rd)
    }
}

#[derive(Debug)]
pub struct Intersection<S1, S2> {
    left: S1,
    right: S2,
}

impl<S1, S2> SignedDistanceFunction for Intersection<S1, S2>
where
    S1: SignedDistanceFunction,
    S2: SignedDistanceFunction,
{
    fn bbox(&self) -> Aabb {
        let bbox = self.left.bbox().intersection(&self.right.bbox());

        match bbox {
            Some(b) => b,
            None => {
                println!(
                    "no intersection between shapes {:?} {:?}",
                    self.left, self.right
                );
                Aabb::new(Vec3::zero())
            }
        }
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.max(rd)
    }
}

#[derive(Debug)]
pub struct Difference<S1, S2> {
    left: S1,
    right: S2,
}

impl<S1, S2> SignedDistanceFunction for Difference<S1, S2>
where
    S1: SignedDistanceFunction,
    S2: SignedDistanceFunction,
{
    fn bbox(&self) -> Aabb {
        self.left.bbox()
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.max(-rd)
    }
}

fn box_intersect(b: &Aabb, r: &Ray) -> (f64, f64) {
    let mut x1 = (b.min().x - r.origin.x) / r.dir.x;
    let mut y1 = (b.min().y - r.origin.y) / r.dir.y;
    let mut z1 = (b.min().z - r.origin.z) / r.dir.z;
    let mut x2 = (b.max().x - r.origin.x) / r.dir.x;
    let mut y2 = (b.max().y - r.origin.y) / r.dir.y;
    let mut z2 = (b.max().z - r.origin.z) / r.dir.z;

    if x1 > x2 {
        std::mem::swap(&mut x1, &mut x2);
    }
    if y1 > y2 {
        std::mem::swap(&mut y1, &mut y2);
    }
    if z1 > z2 {
        std::mem::swap(&mut z1, &mut z2);
    }

    (x1.max(y1).max(z1), x2.min(y2).min(z2))
}
