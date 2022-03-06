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
//!       v   /       \  u
//!         /           \
//!       /               \
//!     /                   \
//!    v                     v
//!
//! ```
//!

mod renderer;
mod spatial_index;

pub use renderer::*;

use geo::{sdf::Sdf, util::arange, Vec3};

/// A Voxel identified by its x, y, z coordinates.
pub type Voxel = (i32, i32, i32);

/// A point in the IJ coordinate space.
pub type IJ = (i32, i32);

/// A point in the XY cartesian plane.
pub type XY = (f64, f64);

/// A line in the cartesian plane.
pub type Line = Vec<XY>;

/// A Scene that can be rendered.
///
/// It's just a collection of Voxels.
#[derive(Debug)]
pub struct Scene {
    voxels: spatial_index::Index,
    sdf_step: f64,
    add: bool,
}

impl Scene {
    /// Create an empty scene.
    pub fn new() -> Self {
        Self {
            voxels: spatial_index::Index::new(),
            sdf_step: 1.0,
            add: true,
        }
    }

    /// Invert the current insertion mode.
    ///
    /// Currently there are two insertion modes:
    ///
    /// - insertion: new voxels are simply added to the scene
    /// - subtraction: new voxels are removed from the scene
    pub fn invert(&mut self) {
        self.add = !self.add;
    }

    /// Iterator over all the voxels in the scene.
    pub fn voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.voxels.iter()
    }

    /// Add the given voxel to the scene.
    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        if self.add {
            self.voxels.add(x, y, z);
        } else {
            self.voxels.remove(x, y, z);
        }
    }

    /// Add a slab parallel to the x axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn xslab(&mut self, (x, y, z): Voxel, (w, hd, hh): (i32, i32, i32)) {
        for dx in 0..w {
            self.aabb((x + dx, y, z), (0, hd, hh));
        }
    }

    /// Add a slab parallel to the y axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn yslab(&mut self, (x, y, z): Voxel, (hw, d, hh): (i32, i32, i32)) {
        for dy in 0..d {
            self.aabb((x, y + dy, z), (hw, 0, hh));
        }
    }

    /// Add a slab parallel to the z axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn zslab(&mut self, (x, y, z): Voxel, (hw, hd, h): (i32, i32, i32)) {
        for dz in 0..h {
            self.aabb((x, y, z + dz), (hw, hd, 0));
        }
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
    pub fn sdf(&mut self, sdf: &impl Sdf) {
        let bbox = sdf.bbox();
        let (tl, br) = (bbox.min(), bbox.max());

        let (x0, y0, z0) = (
            tl.x.round() as i32,
            tl.y.round() as i32,
            tl.z.round() as i32,
        );

        let (s, e) = (tl.floor() + 0.5, br.ceil() - 0.5);

        for (z, zi) in arange(s.z, e.z, self.sdf_step).zip(0..) {
            for (y, yi) in arange(s.y, e.y, self.sdf_step).zip(0..) {
                for (x, xi) in arange(s.x, e.x, self.sdf_step).zip(0..) {
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

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
