use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use rayon::prelude::*;

use geo::{ray::Ray, spatial_index::Intersection, Aabb, Vec3};

use crate::{Camera, Polyline, Scene};

pub struct Settings {
    pub chop_eps: f64,
    pub simplify_eps: f64,
}

pub struct SvgSettings {
    pub width: f64,
    pub height: f64,
    pub stroke_width: f64,
    pub stroke: &'static str,
    pub background: Option<&'static str>,
    pub digits: usize,
}

pub fn render(camera: Camera, scene: &Scene, settings: &Settings) -> Vec<Polyline> {
    // the projection matrix returns points from (-1,-1,-1) to (1,1,1), points
    // outside this area are outside of the clipping region
    let clip_box = Aabb::cube(Vec3::zero(), 2.0);

    let is_visible = |p: Vec3| {
        let d = camera.eye() - p;
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

                if is_visible(p) && clip_box.contains(&projected) {
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

/// Note: The input paths must be in [-1, 1].
pub fn dump_svg(path: &str, paths: &[Polyline], settings: SvgSettings) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    writeln!(
        f,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewbox="0 0 {} {}">"#,
        settings.width, settings.height
    )?;

    if paths.is_empty() {
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

    for path in paths {
        if path.is_empty() {
            continue;
        }

        write!(f, r#"<polyline points=""#)?;
        for p in path.iter() {
            // ignore z value as it is meaningless at this point given that the
            // 3d point has already been projected to a 2d point
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

impl SvgSettings {
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
