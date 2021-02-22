use geo::Aabb;

use crate::{Hit, Ray, Shape, Surface, Vec3};

/// Cylinder positioned at the origin going through the Z axis.
#[derive(Debug, Clone)]
pub struct CylinderGeometry {
    radius: f64,
    zmin: f64,
    zmax: f64,
}

impl CylinderGeometry {
    pub fn new(radius: f64, zrange: (f64, f64)) -> Self {
        let (zmin, zmax) = if zrange.0 < zrange.1 {
            (zrange.0, zrange.1)
        } else {
            (zrange.1, zrange.0)
        };

        CylinderGeometry { zmin, zmax, radius }
    }
}

impl Shape for CylinderGeometry {
    type Intersection = Hit;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        const EPS: f64 = 1e-6;

        let a = ray.dir.x.powi(2) + ray.dir.y.powi(2);
        let b = 2.0 * ray.origin.x * ray.dir.x + 2.0 * ray.origin.y * ray.dir.y;
        let c = ray.origin.x.powi(2) + ray.origin.y.powi(2) - self.radius.powi(2);

        let q = b.powi(2) - 4.0 * a * c;
        if q < EPS {
            return None;
        }

        let s = q.sqrt();
        let mut t0 = (-b + s) / (2.0 * a);
        let mut t1 = (-b - s) / (2.0 * a);
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let z0 = ray.origin.z + t0 * ray.dir.z;
        if t0 > EPS && self.zmin < z0 && self.zmax > z0 {
            return Some(Hit::new(t0, None));
        }

        let z1 = ray.origin.z + t1 * ray.dir.z;
        if t1 > EPS && self.zmin < z1 && self.zmax > z1 {
            return Some(Hit::new(t1, None));
        }

        None
    }

    fn bbox(&self) -> Aabb {
        Aabb::new(Vec3::new(-self.radius, -self.radius, self.zmin)).expanded(Vec3::new(
            self.radius,
            self.radius,
            self.zmax,
        ))
    }
}

impl Surface for CylinderGeometry {
    fn normal_at(&self, mut p: Vec3) -> Vec3 {
        p.z = 0.0;
        p
    }
}
