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

impl Shape for CubeGeometry {
    type Intersection = Hit;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let (tmin, tmax) = self.bbox.ray_intersection(ray)?;
        Some(Hit::new(tmin.min(tmax), None))
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
