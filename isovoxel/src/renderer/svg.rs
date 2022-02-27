use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::Line;

/// Svg settings to use when serializing the scene in Svg.
pub struct SvgSettings<'s> {
    background: Option<&'s str>,
    width: f64,
    height: f64,
    stroke: &'s str,
    stroke_width: f64,
    digits: usize,
    padding: f64,
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

    minx -= settings.padding;
    maxx += settings.padding;
    miny -= settings.padding;
    maxy += settings.padding;

    let (width, height) = (maxx - minx, maxy - miny);

    writeln!(
        f,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{minx} {miny} {width} {height}">"#,
        settings.width, settings.height
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
        settings.stroke,
        settings.stroke_width / f64::max(settings.width / width, settings.height / height)
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

impl<'a> SvgSettings<'a> {
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            background: None,
            width,
            height,
            stroke: "black",
            stroke_width: 1.0,
            digits: 4,
            padding: 0.0,
        }
    }

    pub fn with_background(mut self, background: &'a str) -> Self {
        self.background = Some(background);
        self
    }

    pub fn with_stroke(mut self, stroke: &'a str) -> Self {
        self.stroke = stroke;
        self
    }

    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }
}
