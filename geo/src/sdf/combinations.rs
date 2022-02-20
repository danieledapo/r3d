use crate::{
    mat4::{Mat4, Transform},
    Aabb, Vec3,
};

use super::Sdf;

#[derive(Debug)]
pub struct Union<S1, S2> {
    pub(super) left: S1,
    pub(super) right: S2,
}

#[derive(Debug)]
pub struct Intersection<S1, S2> {
    pub(super) left: S1,
    pub(super) right: S2,
}

#[derive(Debug)]
pub struct Difference<S1, S2> {
    pub(super) left: S1,
    pub(super) right: S2,
}

#[derive(Debug)]
pub struct Translate<S> {
    pub(super) sdf: S,
    pub(super) xlate: Vec3,
}

#[derive(Debug)]
pub struct Transformed<S> {
    pub(super) sdf: S,
    pub(super) matrix: Mat4,
    pub(super) inverse_matrix: Mat4,
}

impl<S1, S2> Sdf for Union<S1, S2>
where
    S1: Sdf,
    S2: Sdf,
{
    fn bbox(&self) -> Aabb {
        self.left.bbox().union(&self.right.bbox())
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.min(rd)
    }
}

impl<S1, S2> Sdf for Intersection<S1, S2>
where
    S1: Sdf,
    S2: Sdf,
{
    fn bbox(&self) -> Aabb {
        let bbox = self.left.bbox().intersection(&self.right.bbox());
        bbox.unwrap_or_else(|| Aabb::new(Vec3::zero()))
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.max(rd)
    }
}

impl<S1, S2> Sdf for Difference<S1, S2>
where
    S1: Sdf,
    S2: Sdf,
{
    fn bbox(&self) -> Aabb {
        self.left.bbox()
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let ld = self.left.dist(p);
        let rd = self.right.dist(p);

        ld.max(-rd)
    }
}

impl<S: Sdf> Sdf for Translate<S> {
    fn dist(&self, p: &Vec3) -> f64 {
        self.sdf.dist(&(*p - self.xlate))
    }

    fn bbox(&self) -> Aabb {
        self.sdf.bbox().translated(self.xlate)
    }
}

impl<S: Sdf> Sdf for Transformed<S> {
    fn bbox(&self) -> Aabb {
        self.sdf.bbox().transform(&self.matrix)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let q = p.transform(&self.inverse_matrix);
        self.sdf.dist(&q)
    }
}
