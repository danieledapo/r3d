//!
//! Isometric voxel renderer implemented from scratch without any dependencies.
//!
//! The renderer only supports one camera positioning that is shown below
//!
//! ```text
//!              ^
//!              |
//!              |  Z
//!              |
//!              |
//!              O
//!             / \
//!       Y   /     \   X
//!         /         \
//!       /             \
//!      v               v
//! ```
//!
//! Where o is the origin and each axis grows positevely along the direction
//! shown above.
//!
//! ## Renderer implementation
//!
//! The renderer works by first triangulating the faces of each Voxel into a set
//! of triangles.
//!
//! ```text
//!
//!        .*.                          .*.
//!      ./   \.                      ./   \.
//!    ./       \.                  ./   |   \.
//!   /           \                /           \
//!  *     top     *              *      |      *
//!  |\           /|              |\           /|
//!  |  \       /  |              |  \   |   /  |
//!  |    \   /    |              |    \   /    |
//!  |      *      |              |      *      |
//!  | left | right|              |    / | \    |
//!  *      |      *              * /    |    \ *
//!   \.    |    ./                \.    |    ./
//!     \.  |  ./                    \.  |  ./
//!       \.|./                        \.|./
//!         *                            *
//!
//! ```
//!
//! The edges of this triangles are marked so that we know which edges are
//! visible and which are not. As you can see from the diagram above, the edge
//! shared by the triangles on the same quadrilateral is never shown.
//!
//! These triangles are then processed from the closest to the farthest,
//! projected into an IJ coordinate space (more details on this later) and drawn
//! into a zbuffer if a triangle was not already there (i.e. draw only the
//! closest triangle). This works because with this projection the triangle is
//! completely visible or completely invisible.
//!
//! The IJ coordinate space is a 2D orthogonal coordinate space defined by
//! integer components used as an intermediate coordinate space to have integer
//! coordinates to easily put the triangles into a hash.
//!
//! ```text
//!
//!      3D space                  IJ space
//!
//!        .*.
//!      ./   \.             +---------+---------+-------->  I
//!    ./       \.           |         |         |
//!   /           \          |         |         |
//!  *     top     *         |   top   |  right  |
//!  |\           /|         |         |         |
//!  |  \       /  |         |         |         |
//!  |    \   /    |         +---------+---------+
//!  |      *      |         |         |
//!  | left | right|         |         |
//!  *      |      *         |  left   |
//!   \.    |    ./          |         |
//!     \.  |  ./            |         |
//!       \.|./              +---------+
//!         *                |
//!                          |
//!                          |
//!                          v J
//! ```
//!
//! Finally, the triangles are transformed and projected into the final XY
//! cartesian plane by simply multiplying the IJ components by the u, v vectors
//! of an isometric grid.
//!
//! ```text
//!
//!               *
//!             /   \
//!       u   /       \  v
//!         /           \
//!       /               \
//!     /                   \
//!    v                     v
//!
//! ```
//!

mod renderer;
pub use renderer::*;

use geo::{sdf::Sdf, util::arange, Vec3};

/// A Voxel identified by its x, y, z coordinates.
pub type Voxel = (i32, i32, i32);

/// A Scene that can be rendered.
///
/// It's just a collection of Voxels.
#[derive(Debug)]
pub struct Scene {
    voxels: Vec<Voxel>,
}

/// Enum over the possible orientations a Triangle can have.
///
/// This can be used to shade each triangle differently.
///
/// ```text
///           top
///             .
///           /   \
///         /       \
///        |  \   /  |
///  left  |    |    |  right
///         \   |   /
///           \ . /
/// ```
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orientation {
    Top,
    Left,
    Right,
}

/// Each Scene is rendered into a collection of Triangle to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triangle<T> {
    pub orientation: Orientation,
    pub pts: [T; 3],

    /// Which segments are visible and which are not.
    ///
    /// Since each face of a voxel is represented as a pair of triangles the
    /// common edge is never visible. Besides, the renderer can also decide to
    /// hide a segment because of its own rendering style. For example, it can
    /// hide an edge if it's shared between two neighboring voxels.
    pub visibility: [bool; 3],
}

impl Scene {
    /// Create an empty scene.
    pub fn new() -> Self {
        Self { voxels: vec![] }
    }

    /// Iterator over all the voxels in the scene.
    pub fn voxels(&self) -> impl Iterator<Item = &Voxel> + '_ {
        self.voxels.iter()
    }

    /// Add the given voxel to the scene.
    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        self.voxels.push((x, y, z));
    }

    /// Add all the voxels included in the box center at the given point with
    /// the given half dimensions.
    pub fn aabb(&mut self, (x, y, z): Voxel, (hw, hd, hh): (i32, i32, i32)) {
        for dz in -hh..=hh {
            for dy in -hd..=hd {
                for dx in -hw..=hw {
                    self.add(x + dx, y + dy, z + dz);
                }
            }
        }
    }

    /// Add all the voxels that are contained in the given sdf by sampling the
    /// sdf by the given step.
    pub fn sdf(&mut self, sdf: &impl Sdf, step: f64) {
        let bbox = sdf.bbox();
        let (tl, br) = (bbox.min(), bbox.max());

        // TODO: here rounding is likely off
        let (x0, y0, z0) = (
            tl.x.round() as i32,
            tl.y.round() as i32,
            tl.z.round() as i32,
        );

        for (z, zi) in arange(tl.z, br.z, step).zip(0..) {
            for (y, yi) in arange(tl.y, br.y, step).zip(0..) {
                for (x, xi) in arange(tl.x, br.x, step).zip(0..) {
                    let d = sdf.dist(&Vec3::new(x, y, z));
                    if d > 0.0 {
                        continue;
                    }

                    self.add(x0 + xi, y0 + yi, z0 + zi);
                }
            }
        }
    }
}

impl<T> Triangle<T> {
    pub fn new(orientation: Orientation, pts: [T; 3], visibility: [bool; 3]) -> Self {
        Self {
            orientation,
            pts,
            visibility,
        }
    }

    pub fn map<TT>(self, f: impl FnMut(T) -> TT) -> Triangle<TT> {
        Triangle {
            pts: self.pts.map(f),
            visibility: self.visibility,
            orientation: self.orientation,
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
