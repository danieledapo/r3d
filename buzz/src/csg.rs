//! Simple [Constructive Solid Geometry][0] framework.
//!
//! [0]: https://en.wikipedia.org/wiki/Constructive_solid_geometry

use geo::ray::Ray;
use geo::spatial_index::Shape;
use geo::{Aabb, Vec3};

use crate::{Hit, Material, Object};

#[derive(Debug, PartialEq, Clone)]
pub struct Csg<G1, G2> {
    geom: CsgGeometry<G1, G2>,
    material: Material,
}

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

impl<G1, G2> Csg<G1, G2> {
    pub fn new(geom: CsgGeometry<G1, G2>, material: Material) -> Self {
        Csg { geom, material }
    }
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

impl<'s, G1, G2> Object<'s> for Csg<G1, G2>
where
    G1: Shape<'s, Intersection = Hit<'s>> + Sync,
    G2: Shape<'s, Intersection = Hit<'s>> + Sync,
{
    fn material(&self) -> &Material {
        &self.material
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.bbox().bounding_sphere()
    }
}

impl<'s, G1, G2> Shape<'s> for Csg<G1, G2>
where
    G1: Shape<'s, Intersection = Hit<'s>>,
    G2: Shape<'s, Intersection = Hit<'s>>,
{
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn intersection(&'s self, ray: &Ray) -> Option<Self::Intersection> {
        Shape::intersection(&self.geom, ray)
    }
}

impl<'s, G1, G2> Shape<'s> for CsgGeometry<G1, G2>
where
    G1: Shape<'s, Intersection = Hit<'s>>,
    G2: Shape<'s, Intersection = Hit<'s>>,
{
    type Intersection = Hit<'s>;

    fn bbox(&self) -> Aabb {
        let mut b = self.left_geom.bbox();
        b.union(&self.right_geom.bbox());
        b
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
