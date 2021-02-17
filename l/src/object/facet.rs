use geo::{primitive::polyline::Polyline, ray::Ray, spatial_index::Shape, Triangle};

use crate::Object;

#[derive(Debug)]
pub struct Facet {
    triangle: Triangle,
}

impl Facet {
    pub fn new(triangle: Triangle) -> Self {
        Self { triangle }
    }
}

impl Shape for Facet {
    type Intersection = f64;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        self.triangle.intersection(ray)
    }

    fn bbox(&self) -> geo::Aabb {
        self.triangle.bbox()
    }
}

impl Object for Facet {
    fn paths(&self) -> Vec<Polyline> {
        vec![vec![
            self.triangle.a,
            self.triangle.b,
            self.triangle.c,
            self.triangle.a,
        ]
        .into()]
    }
}
