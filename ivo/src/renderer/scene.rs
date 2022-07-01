use std::{cmp::Reverse, collections::hash_map::Entry};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{Line, Scene, Voxel, IJ, XY};

use super::{nearness, project_ij, project_iso, IsoTriangle, Orientation};

/// Render the Scene in 3D space into a set of visible lines.
///
/// Note that the lines are simplified and merged together when the endpoints
/// between two segments match in order to reduce the amount of lines.
pub fn render_outlines(scene: &Scene) -> Vec<Line> {
    // store for each position the connectivity as a bitmask (1 vertical, 2
    // u-parallel, 4 j-parallel) so that later we can use this connectivity
    // graph to create straight lines without any duplicate segments.
    let mut connectivity_graph: FxHashMap<IJ, u8> = FxHashMap::default();
    let mut drawn = FxHashSet::default();

    for triangle in render(scene, &mut drawn) {
        for i in 0..triangle.pts.len() {
            let a = triangle.pts[i];
            let b = triangle.pts[(i + 1) % triangle.pts.len()];

            let (a, b) = (a.min(b), a.max(b));

            *connectivity_graph.entry(a).or_default() |= u8::from(triangle.visibility[i]) << i;
            connectivity_graph.entry(b).or_default();
        }
    }

    let mut res = vec![];

    let mut follow_path = |mask: u8, i, j, di, dj| {
        let a = (i, j);
        for d in 0.. {
            // NOTE: *2 is because the triangles are in the
            // "doubled-coordinates" space
            let d = d * 2;

            let b = (i + d * di, j + d * dj);
            match connectivity_graph.get_mut(&b) {
                Some(v) if *v & mask != 0 => {
                    // straight line, follow along
                    *v &= !mask;
                }
                _ if a == b => {
                    // just a single point, skip line
                    break;
                }
                _ => {
                    // a termination point, break line
                    let a = project_iso(a);
                    let b = project_iso(b);

                    res.push(vec![(a.0 / 2.0, a.1 / 2.0), (b.0 / 2.0, b.1 / 2.0)]);
                    break;
                }
            }
        }
    };

    // generate the final paths by following the connections in the connectivity
    // graph
    for t in drawn {
        for (i, j) in t {
            follow_path(1, i, j, 1, 1);
            follow_path(2, i, j, 1, 0);
            follow_path(4, i, j, 0, 1);
        }
    }

    res
}

/// Render the given Scene into a set of IsoTriangle ready to be rendered.
///
/// In particular, each quadrilateral face is broken into two triangles.
///
/// Note that the edges of such triangles are not always visible, be sure to
/// check IsoTriangle::visibility to understand that.
pub fn render_triangles(scene: &Scene) -> Vec<IsoTriangle<XY>> {
    let mut drawn = FxHashSet::default();
    render(scene, &mut drawn)
        .map(|t| {
            t.map(|p| {
                let (a, b) = project_iso(p);
                (a / 2.0, b / 2.0)
            })
        })
        .collect()
}

/// Low-level rendering of a given scene into a list of visible IsoTriangle.
///
/// Note that the edges of such triangles are not always visible, be sure to
/// check IsoTriangle::visibility to understand that.
///
/// Also, the IsoTriangles are in a space where the coordinates have been
/// doubled to avoid having to use floats. When projecting into the cartesian
/// plane be sure to halve them.
///
/// `drawn` contains the coordinates of the rendered triangles in IJ space.
fn render<'a>(
    scene: &'a Scene,
    drawn: &'a mut FxHashSet<[IJ; 3]>,
) -> impl Iterator<Item = IsoTriangle<IJ>> + 'a {
    let mut faces = FxHashMap::default();

    // remove voxels that when projected end up in the same spot,
    // keep only the nearest one.
    for vox in scene.voxels() {
        match faces.entry(project_ij(vox)) {
            Entry::Vacant(v) => {
                v.insert(vox);
            }
            Entry::Occupied(mut o) => {
                if nearness(vox) > nearness(*o.get()) {
                    o.insert(vox);
                }
            }
        }
    }

    // Draw the voxels from closest to farthest, skipping triangles that were
    // already drawn previously by another voxel that, by construction, is on
    // top of the new one.
    let mut voxels = faces.into_values().collect::<Vec<_>>();
    voxels.sort_unstable_by_key(|v| Reverse(nearness(*v)));

    // TODO: this is relatively slow, but fast enough for now...
    let spatial_ix = voxels.iter().copied().collect::<FxHashSet<_>>();

    voxels
        .into_iter()
        .flat_map(move |vox| triangulate(&vox, &spatial_ix))
        .map(|t| t.map(project_ij))
        .filter(move |t| drawn.insert(t.pts))
}

/// Triangulate the left, top and right quadrilateral faces of a given Voxel
/// into a set of triangles.
///
/// This step also calculates which segments are visible or not according to the
/// other voxels in the Scene.
///
/// The triangles are scaled by a factor of 2 (i.e. each voxel is considered to
/// have side length 2 instead of 1) so that each coordinate can be represented
/// as an integer.
///
/// The edges of the triangles are always sorted in the following order:
/// vertical edge, u-parallel edge and v-parallel edge. This sorting can be used
/// for shading.
///
fn triangulate(&(x, y, z): &Voxel, voxels: &FxHashSet<Voxel>) -> [IsoTriangle<Voxel>; 6] {
    let right = voxels.contains(&(x + 1, y, z));
    let front = voxels.contains(&(x, y + 1, z));
    let back = voxels.contains(&(x, y - 1, z));
    let left = voxels.contains(&(x - 1, y, z));
    let up = voxels.contains(&(x, y, z + 1));
    let down = voxels.contains(&(x, y, z - 1));
    let back_right = voxels.contains(&(x + 1, y - 1, z));
    let up_right = voxels.contains(&(x + 1, y, z + 1));
    let down_right = voxels.contains(&(x + 1, y, z - 1));
    let front_down = voxels.contains(&(x, y + 1, z - 1));
    let front_up = voxels.contains(&(x, y + 1, z + 1));
    let front_left = voxels.contains(&(x - 1, y + 1, z));

    // note: scale the voxels to have side length = 2 so that we always work
    // with integer coordinates when in logical space
    let (x, y, z) = (x * 2, y * 2, z * 2);

    [
        IsoTriangle::new(
            Orientation::Top,
            [
                (x - 1, y - 1, z + 1),
                (x + 1, y + 1, z + 1),
                (x - 1, y + 1, z + 1),
            ],
            [false, !front, !left],
        ),
        IsoTriangle::new(
            Orientation::Top,
            [
                (x + 1, y + 1, z + 1),
                (x - 1, y - 1, z + 1),
                (x + 1, y - 1, z + 1),
            ],
            [false, !back, !right],
        ),
        IsoTriangle::new(
            Orientation::Right,
            [
                (x + 1, y - 1, z + 1),
                (x + 1, y - 1, z - 1),
                (x + 1, y + 1, z + 1),
            ],
            [!back || back_right, false, !up || up_right],
        ),
        IsoTriangle::new(
            Orientation::Right,
            [
                (x + 1, y + 1, z - 1),
                (x + 1, y + 1, z + 1),
                (x + 1, y - 1, z - 1),
            ],
            [!front, false, !down || down_right],
        ),
        IsoTriangle::new(
            Orientation::Left,
            [
                (x + 1, y + 1, z + 1),
                (x + 1, y + 1, z - 1),
                (x - 1, y + 1, z - 1),
            ],
            [!right, !down || front_down, false],
        ),
        IsoTriangle::new(
            Orientation::Left,
            [
                (x - 1, y + 1, z - 1),
                (x - 1, y + 1, z + 1),
                (x + 1, y + 1, z + 1),
            ],
            [!left || front_left, !up || front_up, false],
        ),
    ]
}
