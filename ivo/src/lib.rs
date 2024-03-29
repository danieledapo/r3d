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

use geo::{sdf::Sdf, util::arange, v3};

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
    add: bool,
}

impl Scene {
    /// Create an empty scene.
    pub fn new() -> Self {
        Self {
            voxels: spatial_index::Index::new(),
            add: true,
        }
    }

    /// Create an empty scene pre-allocating enough space to cover the bounding
    /// box centered at the origin with the given half dimensions.
    pub fn with_dimensions_hint(width: i32, height: i32, depth: i32) -> Self {
        Self::with_bbox_hint((-width, -height, -depth), (width, height, depth))
    }

    /// Create an empty scene pre-allocating enough space to cover the given
    /// bounding box.
    pub fn with_bbox_hint(min: Voxel, max: Voxel) -> Self {
        Self {
            voxels: spatial_index::Index::with_bbox_hint(min, max),
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

    /// Iterator over the external voxels, that is the voxels that are not
    /// completely enclosed by other voxels.
    pub fn boundary_voxels(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.voxels().filter(|(x, y, z)| {
            [
                (0, 0, 1),
                (0, 0, -1),
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
            ]
            .into_iter()
            .any(|(dx, dy, dz)| !self.is_set(x + dx, y + dy, z + dz))
        })
    }

    /// Add the given voxel to the scene.
    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        if self.add {
            self.voxels.add(x, y, z);
        } else {
            self.voxels.remove(x, y, z);
        }
    }

    /// Check if a given voxel is set or not.
    pub fn is_set(&self, x: i32, y: i32, z: i32) -> bool {
        self.voxels.is_set(x, y, z)
    }

    /// Add a slab parallel to the x axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn xslab(&mut self, (x, y, z): Voxel, (w, hd, hh): (i32, i32, i32)) {
        for dx in 0..=w {
            self.aabb((x + dx, y, z), (0, hd, hh));
        }
    }

    /// Add a slab parallel to the y axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn yslab(&mut self, (x, y, z): Voxel, (hw, d, hh): (i32, i32, i32)) {
        for dy in 0..=d {
            self.aabb((x, y + dy, z), (hw, 0, hh));
        }
    }

    /// Add a slab parallel to the z axis with the given dimensions.
    ///
    /// A slab is just a bounding box, but the instead of giving the center of
    /// the bounding box the starting point of the slab must be provided.
    pub fn zslab(&mut self, (x, y, z): Voxel, (hw, hd, h): (i32, i32, i32)) {
        for dz in 0..=h {
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
    pub fn sdf(&mut self, sdf: &Sdf) {
        let bbox = sdf.bbox();
        let (tl, br) = (bbox.min(), bbox.max());

        let tl = tl.floor();

        // ray march each layer so that we can quickly jump over the outside pixels
        for z in arange(tl.z, br.z, 1.0) {
            for y in arange(tl.y, br.y, 1.0) {
                let mut x = tl.x;
                while x <= br.x {
                    let d = sdf.dist(&v3(x, y, z));
                    if d <= 0.1 {
                        self.add(x.round() as i32, y.round() as i32, z.round() as i32);
                    }

                    x += f64::max(d.round(), 1.0);
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
