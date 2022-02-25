use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::{Line, Orientation, Scene, Triangle, Voxel, IJ, XY};

/// Svg settings to use when serializing the Triangles in Svg.
pub struct SvgSettings<'s> {
    pub background: Option<&'s str>,
    pub scale: f64,
    pub stroke: &'s str,
    pub stroke_width: f64,
    pub digits: usize,
    pub padding: f64,
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

/// Render the Scene in 3D space into a set of Triangles in the cartesian XY
/// plane.
///
/// The returned triangles are always visible, but the edges of such triangles
/// may not be. Be sure to check Triangle::visibility to understand which edges
/// are visible and which are not.
pub fn render(scene: &Scene) -> Vec<Line> {
    let mut faces = HashMap::new();

    // remove voxels that when projected end up in the same spot,
    // keep only the nearest one.
    for &vox in scene.voxels() {
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
    let mut voxels = faces.values().collect::<Vec<_>>();
    voxels.sort_unstable_by_key(|v| Reverse(nearness(**v)));

    // TODO: this is relatively slow, but fast enough for now...
    let spatial_ix = voxels.iter().map(|v| **v).collect::<HashSet<_>>();

    let mut drawn = HashSet::new();

    // store for each position the connectivity as a bitmask (1 vertical, 2
    // u-parallel, 4 j-parallel) so that later we can use this connectivity
    // graph to create straight lines without any duplicate segments.
    let mut connectivity_graph: HashMap<IJ, u8> = HashMap::new();

    for vox in voxels {
        for triangle in triangulate(vox, &spatial_ix) {
            let triangle = triangle.map(project_ij);

            if !drawn.insert(triangle.pts) {
                continue;
            }

            for i in 0..triangle.pts.len() {
                let a = triangle.pts[i];
                let b = triangle.pts[(i + 1) % triangle.pts.len()];

                let (a, b) = (a.min(b), a.max(b));

                *connectivity_graph.entry(a).or_default() |= u8::from(triangle.visibility[i]) << i;
                connectivity_graph.entry(b).or_default();
            }
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

pub fn dump_svg(path: &str, lines: &[Line], settings: &SvgSettings) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    if lines.is_empty() {
        writeln!(
            f,
            r#"<?xml version="1.0" encoding="UTF-8"?>
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 0 0">
    </svg>"#
        )?;
        return Ok(());
    }

    let (mut minx, mut maxx) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut miny, mut maxy) = (f64::INFINITY, f64::NEG_INFINITY);

    for l in lines {
        for &(x, y) in l {
            minx = f64::min(minx, x);
            maxx = f64::max(maxx, x);

            miny = f64::min(miny, y);
            maxy = f64::max(maxy, y);
        }
    }

    minx -= settings.padding + settings.stroke_width;
    maxx += settings.padding + settings.stroke_width;
    miny -= settings.padding + settings.stroke_width;
    maxy += settings.padding + settings.stroke_width;

    let (width, height) = (maxx - minx, maxy - miny);

    writeln!(
        f,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{minx} {miny} {width} {height}">"#,
        width * settings.scale,
        height * settings.scale,
    )?;

    if let Some(background) = settings.background {
        writeln!(
            f,
            r#"<rect x="{minx}" y="{miny}" width="{width}" height="{height}" stroke="none" fill="{background}"/>"#,
        )?;
    }

    // all the lines share the same attributes hence using a group allows to
    // save a lot of space in the final SVG given that such attributes are not
    // repeated.
    writeln!(
        f,
        r#"<g stroke="{}" stroke-width="{}" fill="none">"#,
        settings.stroke, settings.stroke_width
    )?;

    for l in lines {
        write!(f, r#"<polyline points=""#)?;
        for (x, y) in l {
            write!(f, "{x:.digits$},{y:.digits$} ", digits = settings.digits)?;
        }
        writeln!(f, r#"" />"#)?;
    }

    writeln!(f, "</g>\n</svg>")?;

    Ok(())
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
fn triangulate(&(x, y, z): &Voxel, voxels: &HashSet<Voxel>) -> [Triangle<Voxel>; 6] {
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
        Triangle::new(
            Orientation::Top,
            [
                (x - 1, y - 1, z + 1),
                (x + 1, y + 1, z + 1),
                (x - 1, y + 1, z + 1),
            ],
            [false, !front, !left],
        ),
        Triangle::new(
            Orientation::Top,
            [
                (x + 1, y + 1, z + 1),
                (x - 1, y - 1, z + 1),
                (x + 1, y - 1, z + 1),
            ],
            [false, !back, !right],
        ),
        Triangle::new(
            Orientation::Right,
            [
                (x + 1, y - 1, z + 1),
                (x + 1, y - 1, z - 1),
                (x + 1, y + 1, z + 1),
            ],
            [!back || back_right, false, !up || up_right],
        ),
        Triangle::new(
            Orientation::Right,
            [
                (x + 1, y + 1, z - 1),
                (x + 1, y + 1, z + 1),
                (x + 1, y - 1, z - 1),
            ],
            [!front, false, !down || down_right],
        ),
        Triangle::new(
            Orientation::Left,
            [
                (x + 1, y + 1, z + 1),
                (x + 1, y + 1, z - 1),
                (x - 1, y + 1, z - 1),
            ],
            [!right, !down || front_down, false],
        ),
        Triangle::new(
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
