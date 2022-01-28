use geo::{primitive::polyline::Polyline, ray::Ray, spatial_index::Shape, Triangle};

use crate::Object;

#[derive(Debug)]
pub struct Facet {
    triangle: Triangle,
    hatching_lines: u16,
}

impl Facet {
    pub fn new(triangle: Triangle) -> Self {
        Self {
            triangle,
            hatching_lines: 0,
        }
    }

    pub fn with_hatching_lines(mut self, lines: u16) -> Self {
        self.hatching_lines = lines;
        self
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
        let mut paths = vec![self.triangle.boundary()];

        if self.hatching_lines > 0 {
            let Triangle { a, b, c } = self.triangle;

            let &(s, e1, e2) = [(a, b, c), (b, c, a), (c, a, b)]
                .iter()
                .max_by(|(a1, b1, c1), (a2, b2, c2)| {
                    let d1 = f64::from(a1.dist2(*b1) - a1.dist2(*c1));
                    let d2 = f64::from(a2.dist2(*b2) - a2.dist2(*c2));
                    d1.partial_cmp(&d2).unwrap()
                })
                .unwrap();

            let d1 = e1 - s;
            let d2 = e2 - s;

            paths.extend((0..self.hatching_lines).map(|i| {
                let t = f64::from(i + 1) / f64::from(self.hatching_lines + 1);
                Polyline::from(vec![s + d1 * t, s + d2 * t])
            }));
        }

        paths
    }
}
