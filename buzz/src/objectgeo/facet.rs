use geo::{mesh::stl, ray::Ray, spatial_index::Shape, triangle, Aabb, Vec3};

use crate::{Hit, Surface};

#[derive(Debug, PartialEq, Clone)]
pub struct FacetGeometry {
    positions: [Vec3; 3],
    normal: Vec3,
    flat_shading: bool,
}

impl FacetGeometry {
    pub fn new(positions: [Vec3; 3], flat_shading: bool) -> Self {
        let normal = geo::triangle::normal(&positions[0], &positions[1], &positions[2]);

        FacetGeometry {
            positions,
            flat_shading,
            normal,
        }
    }
}
impl Shape for FacetGeometry {
    type Intersection = Hit;

    fn bbox(&self) -> Aabb {
        Aabb::new(self.positions[0])
            .expanded(&self.positions[1])
            .expanded(&self.positions[2])
    }

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let t = triangle::ray_intersection(
            (self.positions[0], self.positions[1], self.positions[2]),
            ray,
        )?;

        Some(Hit::new(t, None))
    }
}

impl Surface for FacetGeometry {
    fn normal_at(&self, pt: Vec3) -> Vec3 {
        if self.flat_shading {
            self.normal
        } else {
            let bary = triangle::barycentric(
                (&self.positions[0], &self.positions[1], &self.positions[2]),
                &pt,
            )
            .expect("point outside triangle");

            let mut n = self.positions[0] * bary.x;
            n += self.positions[1] * bary.y;
            n += self.positions[2] * bary.z;
            n.normalize();

            n
        }
    }
}
