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
