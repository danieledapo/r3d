pub mod aabb;
pub mod mat4;
pub mod plane;
pub mod polyline;
pub mod ray;
pub mod sphere;
pub mod triangle;
pub mod vec3;

pub use vec3::Vec3;

/// An enum over the X, Y and Z axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}
