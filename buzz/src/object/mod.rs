pub mod facet;
pub mod simple_object;

use std::ops::{Deref, DerefMut};

use geo::{
    spatial_index::{Intersection, Shape},
    Vec3,
};

pub use facet::Facet;
pub use simple_object::SimpleObject;

use crate::material::Material;

/// An `Object` that can be rendered.
pub trait Object: Shape<Intersection = Hit> + Surface + Sync + Send {
    /// Getter for the `Material` the `Object` is made of.
    fn material(&self) -> &Material;

    /// Used internally to set a unique id for this object. This is id must be
    /// returned in the `surface_id` field of `Hit` when this `Object` is
    /// intersected.
    fn set_surface_id(&mut self, id: usize);
}

/// A `Surface` is an object that can be shaded.
pub trait Surface: std::fmt::Debug {
    /// Calculate the normal for the given point `p`. This method should never
    /// be called if the `Surface` does not intersect it.
    fn normal_at(&self, p: Vec3) -> Vec3;
}

/// An `Hit` represents an intersection between a `Ray` and the shapes in a
/// `Scene`.
#[derive(Debug)]
pub struct Hit {
    /// `t` parameter wrt the `Ray` that generated this `Hit`
    pub t: f64,

    /// the `Surface` the `Ray` hit
    pub surface_id: usize,

    /// point and normal corresponding at the given `t`. This doesn't need to be
    /// set, but in case they were already calculated as part of the
    /// intersection check a recalculation is avoided this way.
    pub point_and_normal: Option<(Vec3, Vec3)>,
}

impl Hit {
    pub fn new(t: f64, point_and_normal: Option<(Vec3, Vec3)>) -> Self {
        Self {
            t,
            point_and_normal,
            surface_id: 0,
        }
    }
}

impl Intersection for Hit {
    fn t(&self) -> f64 {
        self.t
    }
}

impl<T> Object for Box<T>
where
    T: Object + ?Sized,
{
    fn material(&self) -> &Material {
        self.deref().material()
    }

    fn set_surface_id(&mut self, id: usize) {
        self.deref_mut().set_surface_id(id)
    }
}

impl<T> Surface for Box<T>
where
    T: Surface + ?Sized,
{
    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.deref().normal_at(p)
    }
}
