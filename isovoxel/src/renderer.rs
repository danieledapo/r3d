use std::{
    cmp::Reverse,
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::{Orientation, Scene, Triangle, Voxel, IJ, XY};

pub struct SvgSettings<'s> {
    pub background: Option<&'s str>,
    pub scale: f64,
    pub stroke: &'s str,
    pub stroke_width: f64,
    pub digits: usize,
    pub padding: f64,
}

fn project_ij((x, y, z): Voxel) -> IJ {
    (x - z, y - z)
}

fn project_iso((i, j): IJ) -> XY {
    let (i, j) = (f64::from(i), f64::from(j));

    // even though these aren't marked const (especially since sqrt is not
    // const) the compiler is smarter enough to replace the calls with just the
    // constant
    let dx = f64::sqrt(3.0) / 2.0;
    let dy: f64 = 0.5;

    (i * dx - j * dx, i * dy + j * dy)
}

fn nearness((x, y, z): Voxel) -> i32 {
    x + y + z
}

pub fn render(scene: &Scene) -> Vec<Triangle<XY>> {
    let mut faces = HashMap::new();

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

    let mut voxels = faces.values().collect::<Vec<_>>();
    voxels.sort_unstable_by_key(|v| Reverse(nearness(**v)));

    // TODO: this is relatively slow, but fast enough for now...
    let spatial_ix = voxels.iter().map(|v| **v).collect::<HashSet<_>>();

    let mut drawn = HashSet::new();
    let mut res = vec![];

    for vox in voxels {
        for triangle in triangulate(vox, &spatial_ix) {
            let triangle = triangle.map(project_ij);

            let mut k = triangle.pts;
            k.sort_unstable();
            if !drawn.insert(k) {
                continue;
            }

            res.push(triangle.map(|ij| {
                let (x, y) = project_iso(ij);
                (x / 2.0, y / 2.0)
            }));
        }
    }

    res
}

pub fn dump_svg(path: &str, triangles: &[Triangle<XY>], settings: &SvgSettings) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    if triangles.is_empty() {
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

    for t in triangles {
        for (x, y) in t.pts {
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

    for triangle in triangles {
        for i in 0..3 {
            if !triangle.visibility[i] {
                continue;
            }

            let (ax, ay) = triangle.pts[i];
            let (bx, by) = triangle.pts[(i + 1) % 3];

            writeln!(
                f,
                r#"<polyline points="{ax:.digits$},{ay:.digits$} {bx:.digits$},{by:.digits$}" />"#,
                digits = settings.digits
            )?;
        }
    }

    writeln!(f, "</g>\n</svg>")?;

    Ok(())
}

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
                (x + 1, y - 1, z + 1),
                (x - 1, y - 1, z + 1),
            ],
            [!right, !back, false],
        ),
        Triangle::new(
            Orientation::Right,
            [
                (x + 1, y - 1, z - 1),
                (x + 1, y - 1, z + 1),
                (x + 1, y + 1, z + 1),
            ],
            [!back || back_right, !up || up_right, false],
        ),
        Triangle::new(
            Orientation::Right,
            [
                (x + 1, y + 1, z + 1),
                (x + 1, y - 1, z - 1),
                (x + 1, y + 1, z - 1),
            ],
            [false, !down || down_right, !front],
        ),
        Triangle::new(
            Orientation::Left,
            [
                (x - 1, y + 1, z - 1),
                (x + 1, y + 1, z - 1),
                (x + 1, y + 1, z + 1),
            ],
            [!down || front_down, !right, false],
        ),
        Triangle::new(
            Orientation::Left,
            [
                (x + 1, y + 1, z + 1),
                (x - 1, y + 1, z + 1),
                (x - 1, y + 1, z - 1),
            ],
            [!up || front_up, !left || front_left, false],
        ),
    ]
}
