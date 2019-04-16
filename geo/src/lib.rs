pub mod aabb;
pub mod ray;
pub mod sphere;
pub mod stl;
pub mod triangle;
pub mod vec3;

/// An enum over the X, Y and Z axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
