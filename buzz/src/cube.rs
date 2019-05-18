use geo::Aabb;

use crate::{Hit, Ray, Shape, Surface, Vec3};

#[derive(Debug, Clone)]
pub struct CubeGeometry {
    bbox: Aabb,
}

impl CubeGeometry {
    pub fn new(bbox: Aabb) -> Self {
        CubeGeometry { bbox }
    }
}

impl<'s> Shape<'s> for CubeGeometry {
    type Intersection = Hit<'s>;

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        let min = *self.bbox().min();
        let max = *self.bbox().max();

        let mut tmin = (min.x - ray.origin.x) / ray.dir.x;
        let mut tmax = (max.x - ray.origin.x) / ray.dir.x;
        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (min.y - ray.origin.y) / ray.dir.y;
        let mut tymax = (max.y - ray.origin.y) / ray.dir.y;
        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if tmin > tymax || tymin > tmax {
            return None;
        }

        tmin = tmin.max(tymin);
        tmax = tmax.min(tymax);

        let mut tzmin = (min.z - ray.origin.z) / ray.dir.z;
        let mut tzmax = (max.z - ray.origin.z) / ray.dir.z;
        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if tmin > tzmax || tzmin > tmax {
            return None;
        }

        tmin = tmin.max(tzmin);
        tmax = tmax.min(tzmax);

        Some(Hit {
            t: tmin.min(tmax),
            surface: self,
        })
    }

    fn bbox(&self) -> Aabb {
        self.bbox.clone()
    }
}

impl Surface for CubeGeometry {
    fn normal_at(&self, p: Vec3) -> Vec3 {
        const EPS: f64 = 1e-6;

        let min = self.bbox.min();
        let max = self.bbox.max();

        if p.x < min.x + EPS {
            return Vec3::new(-1.0, 0.0, 0.0);
        }
        if p.x > max.x - EPS {
            return Vec3::new(1.0, 0.0, 0.0);
        }

        if p.y < min.y + EPS {
            return Vec3::new(0.0, -1.0, 0.0);
        }
        if p.y > max.y - EPS {
            return Vec3::new(0.0, 1.0, 0.0);
        }

        if p.z < min.z + EPS {
            return Vec3::new(0.0, 0.0, -1.0);
        }
        if p.z > max.z - EPS {
            return Vec3::new(0.0, 0.0, 1.0);
        }

        Vec3::new(0.0, 1.0, 0.0)
    }
}
