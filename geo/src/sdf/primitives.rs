use crate::{sphere, Aabb, Vec3};

use super::Sdf;

#[derive(Debug, Clone)]
pub struct Sphere {
    radius: f64,
}

#[derive(Debug, Clone)]
pub struct Cuboid {
    size: Vec3,
}

#[derive(Debug, Clone)]
pub struct Cylinder {
    radius: f64,
    height: f64,
}

#[derive(Debug, Clone)]
pub struct Torus {
    r1: f64,
    r2: f64,
}

#[derive(Debug, Clone)]
pub struct Capsule {
    a: Vec3,
    b: Vec3,
    r: f64,
}

#[derive(Debug, Clone)]
pub struct Octahedron {
    r: f64,
}

impl Sphere {
    pub fn new(radius: f64) -> Self {
        Sphere { radius }
    }
}

impl Cuboid {
    pub fn new(size: Vec3) -> Self {
        Cuboid { size }
    }
}

impl Cylinder {
    pub fn new(radius: f64, height: f64) -> Self {
        Cylinder { radius, height }
    }
}

impl Torus {
    pub fn new(r1: f64, r2: f64) -> Self {
        Self { r1, r2 }
    }
}

impl Capsule {
    pub fn new(a: Vec3, b: Vec3, r: f64) -> Self {
        Self { a, b, r }
    }
}

impl Octahedron {
    pub fn new(r: f64) -> Self {
        Self { r }
    }
}

impl Sdf for Sphere {
    fn bbox(&self) -> Aabb {
        sphere::bounding_box(Vec3::zero(), self.radius)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        p.norm() - self.radius
    }
}

impl Sdf for Cuboid {
    fn bbox(&self) -> Aabb {
        let d = self.size / 2.0;
        Aabb::new(-d).expanded(d)
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

impl Sdf for Cylinder {
    fn bbox(&self) -> Aabb {
        Aabb::new(Vec3::new(-self.radius, -self.height / 2.0, -self.radius)).expanded(Vec3::new(
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

impl Sdf for Torus {
    fn dist(&self, p: &Vec3) -> f64 {
        let q = Vec3::new(Vec3::new(p.x, p.y, 0.0).norm() - self.r2, p.z, 0.0);
        q.norm() - self.r1
    }

    fn bbox(&self) -> Aabb {
        let a = self.r1;
        let b = self.r1 + self.r2;

        Aabb::new(Vec3::new(-b, -b, -a)).expanded(Vec3::new(b, b, a))
    }
}

impl Sdf for Capsule {
    fn dist(&self, p: &Vec3) -> f64 {
        let pa = *p - self.a;
        let ba = self.b - self.a;
        let h = f64::clamp(pa.dot(ba) / ba.dot(ba), 0.0, 1.0);
        (pa - ba * h).norm() - self.r
    }

    fn bbox(&self) -> Aabb {
        Aabb::new(self.a - self.r)
            .expanded(self.a + self.r)
            .expanded(self.b - self.r)
            .expanded(self.b + self.r)
    }
}

impl Sdf for Octahedron {
    fn dist(&self, p: &Vec3) -> f64 {
        let m = p.x.abs() + p.y.abs() + p.z.abs() - self.r;
        m * f64::to_radians(30.0).tan()
    }

    fn bbox(&self) -> Aabb {
        Aabb::new(Vec3::replicate(-self.r)).expanded(Vec3::replicate(self.r))
    }
}
