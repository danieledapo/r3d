use geo::{sdf::Sdf, util::arange, Vec3};

mod renderer;
pub use renderer::*;

pub type Voxel = (i32, i32, i32);
pub type IJ = (i32, i32);
pub type XY = (f64, f64);

#[derive(Debug)]
pub struct Scene {
    voxels: Vec<Voxel>,
}

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

impl Scene {
    pub fn new() -> Self {
        Self { voxels: vec![] }
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        self.voxels.push((x, y, z));
    }

    pub fn aabb(&mut self, (x, y, z): Voxel, (hw, hd, hh): (i32, i32, i32)) {
        for dz in -hh..=hh {
            for dy in -hd..=hd {
                for dx in -hw..=hw {
                    self.add(x + dx, y + dy, z + dz);
                }
            }
        }
    }

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

    pub fn voxels(&self) -> impl Iterator<Item = &Voxel> + '_ {
        self.voxels.iter()
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
