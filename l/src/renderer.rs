use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use rayon::prelude::*;

use geo::{ray::Ray, spatial_index::Intersection, Aabb, Vec3};

use crate::{Camera, Polyline, Scene};

/// Simple struct to hold the rendering params together.
#[derive(Debug, PartialEq, Clone)]
pub struct Settings {
    /// the epsilon used to sample the paths in the scene to check for
    /// visibility.
    pub chop_eps: f64,

    /// the epsilon used to simplify the lines after having checked for point
    /// visibility.
    pub simplify_eps: f64,
}

/// The settings to render a set of `Polyline`s as returned by `render` to a
/// SVG.
///
/// Note that the input `Polyline`s are automatically scaled and translated so
/// that their bounding box matches the SVG viewport of the given `width` and
/// `height`.
#[derive(Debug, PartialEq, Clone)]
pub struct SvgSettings<'s> {
    /// width of the SVG viewbox
    pub width: f64,

    /// height of the viewbox
    pub height: f64,

    /// stroke width of all the lines
    pub stroke_width: f64,

    /// stroke color of all the lines
    pub stroke: &'s str,

    /// optional background color of the SVG
    pub background: Option<&'s str>,

    /// how many digits to keep in the floating point numbers dumped to the SVG
    pub digits: usize,
}

/// Render the given `Scene` using the given `Camera` and `Settings`.
pub fn render(camera: &Camera, scene: &Scene, settings: &Settings) -> Vec<Polyline> {
    // the projection matrix returns points from (-1,-1,-1) to (1,1,1), points
    // outside this area are outside of the clipping region
    let clip_box = Aabb::cuboid(Vec3::zero(), 2.0);

    let is_visible = |p: Vec3| {
        let d = camera.position() - p;
        let ray = Ray::new(p + d * settings.chop_eps, d);

        match scene.intersection(&ray) {
            None => true,
            Some((_, t)) => t.t() >= d.norm(),
        }
    };

    let paths: Vec<_> = scene.objects.iter().flat_map(|o| o.paths()).collect();

    paths
        .par_iter()
        .filter(|p| !p.is_empty())
        .flat_map(|path: &Polyline| {
            let mut out = vec![];

            let mut cur = Polyline::new();
            for p in path.chop(settings.chop_eps).iter() {
                let projected = camera.project(p);

                if clip_box.contains(&projected) && is_visible(p) {
                    cur.push(projected);
                } else if !cur.is_empty() {
                    out.push(cur.simplified(settings.simplify_eps));
                    cur = Polyline::new();
                }
            }

            if !cur.is_empty() {
                out.push(cur.simplified(settings.simplify_eps));
            }

            out
        })
        .collect()
}

/// Dump to `path` the given `Polyline`s with the given settings.
///
/// Note: The input `Polyline`s must be in [-1, 1].
pub fn dump_svg(path: &str, poylines: &[Polyline], settings: SvgSettings) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    writeln!(
        f,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}">"#,
        settings.width, settings.height
    )?;

    if poylines.is_empty() {
        return writeln!(f, "</svg>");
    }

    if let Some(background) = settings.background {
        writeln!(
            f,
            r#"<rect x="0" y="0" width="{:.digits$}" height="{:.digits$}" stroke="none" fill="{}"/>"#,
            settings.width,
            settings.height,
            background,
            digits = settings.digits
        )?;
    }

    // all the lines share the same attributes hence using a group allows to
    // save a lot of space in the final SVG given that such attributes are not
    // repeated.
    writeln!(
        f,
        r#"<g stroke="{}" stroke-width="{}" fill="none" >"#,
        settings.stroke, settings.stroke_width
    )?;

    // subtract the stroke width from the available dimensions so that the
    // rendered lines are all inside the requested dimensions
    let w2 = (settings.width - settings.stroke_width) / 2.0;
    let h2 = (settings.height - settings.stroke_width) / 2.0;

    for path in poylines {
        if path.is_empty() {
            continue;
        }

        write!(f, r#"<polyline points=""#)?;
        for mut p in path.iter() {
            // invert y coordinate because in world space (0, 0) lies at the
            // center and the y axis grows upwards while in image space (0, 0)
            // is at the top left and y grows downwards.
            p.y = -p.y;

            // ignore z value as it is meaningless at this point given that the
            // 3d point has already been projected to a 2d point.
            write!(
                f,
                "{:.digits$},{:.digits$} ",
                (p.x + 1.0) * w2,
                (p.y + 1.0) * h2,
                digits = settings.digits
            )?;
        }
        writeln!(f, r#"" />"#)?;
    }

    writeln!(f, "</g>\n</svg>")?;

    Ok(())
}

impl SvgSettings<'_> {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            stroke_width: 1.0,
            stroke: "black",
            background: Some("white"),
            digits: 3,
        }
    }
}
