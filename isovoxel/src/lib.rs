mod renderer;
pub use renderer::*;

pub type P3 = (i32, i32, i32);
pub type IJ = (i32, i32);
pub type XY = (f64, f64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Voxel(P3);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Orientation {
    Top,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Triangle<T> {
    pub orientation: Orientation,
    pub pts: [T; 3],
    pub visibility: [bool; 3],
}

impl Voxel {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self((x, y, z))
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
