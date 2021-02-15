use geo::{spatial_index::Shape, triangle, Vec3};

use crate::Object;

#[derive(Debug)]
pub struct Facet {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

impl Facet {
    pub fn new(triangle: [Vec3; 3]) -> Self {
        Self {
            a: triangle[0],
            b: triangle[1],
            c: triangle[2],
        }
    }
}

impl Shape for Facet {
    type Intersection = f64;

    fn intersection(&self, ray: &geo::ray::Ray) -> Option<Self::Intersection> {
        triangle::ray_intersection((self.a, self.b, self.c), ray)
    }

    fn bbox(&self) -> geo::Aabb {
        triangle::bbox(self.a, self.b, self.c)
    }
}

impl Object for Facet {
    fn paths(&self) -> Vec<crate::Polyline> {
        vec![vec![self.a, self.b, self.c]]
    }
}
