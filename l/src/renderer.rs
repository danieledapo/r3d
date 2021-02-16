use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use rayon::prelude::*;

use geo::{ray::Ray, spatial_index::Intersection, Aabb, Vec3};

use crate::{Camera, Polyline, Scene};

pub struct Settings {
    pub eps: f64,
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
        let ray = Ray::new(p + d * settings.eps, d);

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

            let mut cur = vec![];
            for (&s, &e) in path.iter().zip(path.iter().skip(1)) {
                let dir = (e - s).normalized();
                let maxl = (e - s).norm();

                let prev_size = cur.len();
                let mut l = 0.0;
                loop {
                    let p = s + dir * l;

                    let projected = camera.project(p);

                    if is_visible(p) && clip_box.contains(&projected) {
                        if cur.len() - prev_size > 1 {
                            cur.pop();
                        }
                        cur.push(projected);
                    } else if !cur.is_empty() {
                        out.push(cur);
                        cur = vec![];
                    }

                    l += settings.eps;
                    if l > maxl {
                        break;
                    }
                }
            }

            if !cur.is_empty() {
                out.push(cur);
            }

            out
        })
        .collect()
}

/// Note: The input paths must be in [-1, 1].
pub fn dump_svg(path: &str, paths: &[Polyline], settings: SvgSettings) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    writeln!(f, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

    if paths.is_empty() {
        return Ok(());
    }

    writeln!(
        f,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewbox="0 0 {} {}">"#,
        settings.width, settings.height
    )?;

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

    for path in paths {
        if path.is_empty() {
            continue;
        }

        write!(f, r#"<polyline points=""#)?;
        for p in path {
            // ignore z value as it is meaningless at this point
            write!(
                f,
                "{:.digits$},{:.digits$} ",
                (p.x + 1.0) * settings.width / 2.0,
                (p.y + 1.0) * settings.height / 2.0,
                digits = settings.digits
            )?;
        }
        writeln!(
            f,
            r#"" fill="none" stroke="{}" stroke-width="{}" />"#,
            settings.stroke, settings.stroke_width
        )?;
    }

    writeln!(f, "</svg>")?;

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
