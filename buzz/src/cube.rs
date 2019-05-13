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
        let n = (*self.bbox.min() - ray.origin) / ray.dir;
        let f = (*self.bbox.max() - ray.origin) / ray.dir;

        let ab = Aabb::new(n).expanded(&f);
        let n = ab.min();
        let f = ab.max();

        let t0 = n.x.max(n.y).max(n.z);
        let t1 = f.x.min(f.y).min(f.z);

        if t0 > 0.0 && t0 < t1 {
            Some(Hit {
                t: t0,
                surface: self,
            })
        } else {
            None
        }
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
