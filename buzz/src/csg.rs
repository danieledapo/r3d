//! Simple [Constructive Solid Geometry][0] framework.
//!
//! [0]: https://en.wikipedia.org/wiki/Constructive_solid_geometry

use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::Aabb;

use crate::Hit;

#[derive(Debug, PartialEq, Clone)]
pub struct CsgGeometry<G1, G2> {
    left_geom: G1,
    right_geom: G2,
    op: CsgOp,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CsgOp {
    Union,
    Difference,
    Intersection,
}

impl<G1, G2> CsgGeometry<G1, G2> {
    pub fn new(left_geom: G1, right_geom: G2, op: CsgOp) -> Self {
        CsgGeometry {
            left_geom,
            right_geom,
            op,
        }
    }

    pub fn union<G3>(self, right_geom: G3) -> CsgGeometry<Self, G3> {
        CsgGeometry::new(self, right_geom, CsgOp::Union)
    }

    pub fn difference<G3>(self, right_geom: G3) -> CsgGeometry<Self, G3> {
        CsgGeometry::new(self, right_geom, CsgOp::Difference)
    }

    pub fn intersection<G3>(self, right_geom: G3) -> CsgGeometry<Self, G3> {
        CsgGeometry::new(self, right_geom, CsgOp::Intersection)
    }
}

impl<'s, G1, G2> Shape<'s> for CsgGeometry<G1, G2>
where
    G1: Shape<'s, Intersection = Hit<'s>>,
    G2: Shape<'s, Intersection = Hit<'s>>,
{
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        match self.op {
            CsgOp::Union => {
                let mut b = self.left_geom.bbox();
                b.union(&self.right_geom.bbox());
                b
            }
            CsgOp::Intersection => self
                .left_geom
                .bbox()
                .intersection(&self.right_geom.bbox())
                .expect(
                r#"please do not try to build a CsgOp::Intersection between non intersecting shapes,
                   I know this method shouldn't blow up, but eh. The correct thing would be to avoid
                   creating empty intersections
                "#,
            ),
            CsgOp::Difference => {
                // TODO: I'm not too sure about this. I mean, it is correct
                // because by definition the difference between two objects
                // cannot return a bigger object than the first, but probably we
                // can reduce the bbox further.
                self.left_geom.bbox()
            }
        }
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        match self.op {
            CsgOp::Union => {
                let hit1 = self.left_geom.intersection(ray);
                let hit2 = self.right_geom.intersection(ray);

                match (hit1, hit2) {
                    (None, None) => None,
                    (None, Some(h)) | (Some(h), None) => Some(h),
                    (Some(h1), Some(h2)) => {
                        if h1.t < h2.t {
                            Some(h1)
                        } else {
                            Some(h2)
                        }
                    }
                }
            }
            CsgOp::Intersection => self
                .left_geom
                .intersection(ray)
                .and_then(|_| self.right_geom.intersection(ray)),
            CsgOp::Difference => self.left_geom.intersection(ray).and_then(|h| {
                if self.right_geom.intersection(ray).is_some() {
                    None
                } else {
                    Some(h)
                }
            }),
        }
    }
}
