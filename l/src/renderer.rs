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

pub fn render(camera: Camera, scene: &Scene, settings: &Settings) -> Vec<Polyline> {
    // from (-1,-1,-1) to (1,1,1)
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

            for (&s, &e) in path.iter().zip(path.iter().cycle().skip(1)) {
                let mut path = vec![];

                let dir = (e - s).normalized();
                let maxl = (e - s).norm();

                let mut l = 0.0;
                loop {
                    let p = s + dir * l;

                    let projected = camera.project(p);

                    if is_visible(p) && clip_box.contains(&projected) {
                        if path.len() > 1 {
                            path.pop();
                        }
                        path.push(projected);
                    } else if !path.is_empty() {
                        out.push(path);
                        path = vec![];
                    }

                    l += settings.eps;
                    if l > maxl {
                        break;
                    }
                }

                if !path.is_empty() {
                    out.push(path);
                }
            }

            out
        })
        .collect()
}

pub fn dump_svg(path: &str, paths: &[Polyline], (width, height): (f64, f64)) -> io::Result<()> {
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    writeln!(f, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;

    if paths.is_empty() {
        return Ok(());
    }

    writeln!(f, r#"<svg viewbox="0 0 {} {}">"#, width, height)?;
    writeln!(
        f,
        r#"<rect x="0" y="0" width="{}" height="{}" stroke="none" stroke-width="0.01" fill="white"/>"#,
        width, height
    )?;

    for path in paths {
        if path.is_empty() {
            continue;
        }

        write!(f, r#"<polyline points=""#)?;
        for p in path {
            // ignore z value as it is meaningless at this point
            write!(
                f,
                "{},{} ",
                (p.x + 1.0) * width / 2.0,
                (p.y + 1.0) * height / 2.0,
            )?;
        }
        writeln!(f, r#"" fill="none" stroke="black" />"#)?;
    }

    writeln!(f, "</svg>")?;

    Ok(())
}
