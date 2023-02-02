//! Dynamic spatial index optimized for voxels.
//!
//! It uses a quite dumb, but efficient enough system where each 1 million
//! voxels added to the index it creates the complete grid covering the bounding
//! box of the voxels in the scene.
//!
//! This is to prevent having a huge hash-set which gets progressively slower
//! because of collisions. Besides, indexing into a grid should be definitely
//! faster (and more memory efficient) than an hash-set.
//!
//! The 1 million threshold is arbitrary, but it should be big enough to avoid
//! rebuilding the grid too many times which is quite costly and FxHashSet is
//! quite fast anyway.
//!
//! This is definitely not a good general solution, but it's quite easy to
//! implement.

use crate::Voxel;

use rustc_hash::FxHashSet;

pub const MAX_RUNNING_VOXELS: usize = 1_000_000;

#[derive(Debug)]
pub struct Index {
    grid: Option<Grid>,
    outside_grid: FxHashSet<Voxel>,
    min: Voxel,
    max: Voxel,
}

impl Index {
    pub fn new() -> Self {
        Self {
            grid: None,
            outside_grid: FxHashSet::default(),
            min: (i32::MAX, i32::MAX, i32::MAX),
            max: (i32::MIN, i32::MIN, i32::MIN),
        }
    }

    pub fn with_bbox_hint(min: Voxel, max: Voxel) -> Self {
        Self {
            grid: Some(Grid::new(min, max)),
            outside_grid: FxHashSet::default(),
            min,
            max,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.outside_grid
            .iter()
            .copied()
            .chain(self.grid.as_ref().map(|g| g.iter()).into_iter().flatten())
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        match &mut self.grid {
            Some(grid) if grid.covers_cell(x, y, z) => {
                grid.add(x, y, z);
            }
            _ => {
                self.min = (
                    i32::min(x, self.min.0),
                    i32::min(y, self.min.1),
                    i32::min(z, self.min.2),
                );
                self.max = (
                    i32::max(x, self.max.0),
                    i32::max(y, self.max.1),
                    i32::max(z, self.max.2),
                );
                self.outside_grid.insert((x, y, z));

                if self.outside_grid.len() >= MAX_RUNNING_VOXELS {
                    let mut grid = Grid::new(self.min, self.max);
                    for (x, y, z) in self.outside_grid.drain() {
                        grid.add(x, y, z);
                    }

                    if let Some(g) = &self.grid {
                        for (x, y, z) in g.iter() {
                            grid.add(x, y, z);
                        }
                    }

                    self.grid = Some(grid);
                }
            }
        }
    }

    pub fn remove(&mut self, x: i32, y: i32, z: i32) {
        match &mut self.grid {
            Some(grid) if grid.covers_cell(x, y, z) => {
                grid.remove(x, y, z);
            }
            _ => {
                self.outside_grid.remove(&(x, y, z));
            }
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    min: Voxel,
    max: Voxel,
    cells: Vec<u64>,
}

impl Grid {
    pub fn new(min: Voxel, max: Voxel) -> Self {
        assert!(min <= max);

        let (w, h, d) = distance(min, max);
        let size = (w * h * d + 63) / 64;

        Self {
            min,
            max,
            cells: vec![0; size],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Voxel> + '_ {
        self.cells
            .iter()
            .enumerate()
            .flat_map(move |(p, &c)| ones(c).map(move |bi| self.rev_index(p, bi as usize)))
    }

    pub fn covers_cell(&self, x: i32, y: i32, z: i32) -> bool {
        (self.min.0..=self.max.0).contains(&x)
            && (self.min.1..=self.max.1).contains(&y)
            && (self.min.2..=self.max.2).contains(&z)
    }

    pub fn add(&mut self, x: i32, y: i32, z: i32) {
        let (p, bi) = self.index(x, y, z);
        self.cells[p] |= 1 << bi;
    }

    pub fn remove(&mut self, x: i32, y: i32, z: i32) {
        let (p, bi) = self.index(x, y, z);
        self.cells[p] &= !(1 << bi);
    }

    fn index(&self, x: i32, y: i32, z: i32) -> (usize, usize) {
        let (x, y, z) = (x - self.min.0, y - self.min.1, z - self.min.2);
        let (x, y, z) = (x as usize, y as usize, z as usize);

        let (w, h, _d) = distance(self.min, self.max);
        let i = z * w * h + y * w + x;

        (i / 64, i % 64)
    }

    fn rev_index(&self, p: usize, bi: usize) -> Voxel {
        let (w, h, _d) = distance(self.min, self.max);

        let i = p * 64 + bi;

        let z = i / (w * h);
        let y = (i % (w * h)) / w;
        let x = (i % (w * h)) % w;

        (
            self.min.0 + x as i32,
            self.min.1 + y as i32,
            self.min.2 + z as i32,
        )
    }
}

fn ones(mut n: u64) -> impl Iterator<Item = u32> {
    let mut bi = 0;

    std::iter::from_fn(move || {
        let z = n.trailing_zeros();
        if z == 64 {
            return None;
        }

        bi += z + 1;
        n = if z == 63 { 0 } else { n >> (z + 1) };
        Some(bi - 1)
    })
}

fn distance(a: Voxel, b: Voxel) -> (usize, usize, usize) {
    let (w, h, d) = (b.0 - a.0 + 1, b.1 - a.1 + 1, b.2 - a.2 + 1);
    (w as usize, h as usize, d as usize)
}
