use crate::{Voxel, IJ, XY};

mod line_renderer;
mod svg;

pub use line_renderer::render;
pub use svg::{dump_svg, SvgSettings};

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
enum Orientation {
    Top,
    Left,
    Right,
}

/// Each Scene is rendered into a collection of Triangle to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IsoTriangle<T> {
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

/// Project the given Voxel in 3D space to the IJ coordinate space.
fn project_ij((x, y, z): Voxel) -> IJ {
    (x - z, y - z)
}

/// Project the given point in IJ space to the final XY cartesian plane.
fn project_iso((i, j): IJ) -> XY {
    let (i, j) = (f64::from(i), f64::from(j));

    // even though these aren't marked const (especially since sqrt is not
    // const) the compiler is smarter enough to replace the calls with just the
    // constant
    let dx = f64::sqrt(3.0) / 2.0;
    let dy: f64 = 0.5;

    (i * dx - j * dx, i * dy + j * dy)
}

/// Return a nearness score for the given voxel.
///
/// The higher the value the closest the voxel.
fn nearness((x, y, z): Voxel) -> i32 {
    x + y + z
}

impl<T> IsoTriangle<T> {
    pub fn new(orientation: Orientation, pts: [T; 3], visibility: [bool; 3]) -> Self {
        Self {
            orientation,
            pts,
            visibility,
        }
    }

    pub fn map<TT>(self, f: impl FnMut(T) -> TT) -> IsoTriangle<TT> {
        IsoTriangle {
            pts: self.pts.map(f),
            visibility: self.visibility,
            orientation: self.orientation,
        }
    }
}
