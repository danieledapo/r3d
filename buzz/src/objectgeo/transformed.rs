use geo::{mat4::Mat4, Aabb};

use crate::{Hit, Ray, Shape, Surface, Vec3};

#[derive(Debug, PartialEq, Clone)]
pub struct TransformedGeometry<S> {
    shape: S,
    trans: Mat4,
    inverse_trans: Mat4,
}

impl<S> TransformedGeometry<S> {
    pub fn new(shape: S, trans: Mat4) -> Self {
        let inverse_trans = trans.inverse();

        TransformedGeometry {
            shape,
            trans,
            inverse_trans,
        }
    }
}

impl<S> Shape for TransformedGeometry<S>
where
    S: Shape<Intersection = Hit> + Surface,
{
    type Intersection = Hit;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let transformed_ray = ray.clone() * &self.inverse_trans;
        let hit = self.shape.intersection(&transformed_ray)?;

        let p = transformed_ray.point_at(hit.t);
        let n = self.shape.normal_at(p);

        let intersection = p * &self.trans;
        let tn = self.inverse_trans.transform_normal(&n);

        Some(Hit::new((p - ray.origin).norm(), Some((intersection, tn))))
    }

    fn bbox(&self) -> Aabb {
        self.shape.bbox() * &self.trans
    }
}

impl<S> Surface for TransformedGeometry<S>
where
    S: Surface,
{
    fn normal_at(&self, _p: Vec3) -> Vec3 {
        unreachable!()
    }
}
