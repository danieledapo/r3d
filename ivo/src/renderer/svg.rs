use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use crate::{IsoTriangle, Line, Orientation, XY};

/// Svg settings to use when serializing the scene in Svg.
pub struct SvgSettings<'s> {
    background: Option<&'s str>,
    width: f64,
    height: f64,
    stroke: &'s str,
    stroke_width: f64,
    digits: usize,
    padding: f64,

    fill_colors: [Option<&'s str>; 3],
}

pub fn dump_outlines_svg(path: &str, lines: &[Line], settings: &SvgSettings) -> io::Result<()> {
    svg_prelude(
        path,
        settings,
        || lines.iter().flat_map(|l| l.iter().copied()),
        |f, stroke_width| {
            // all the lines share the same attributes hence using a group allows to
            // save a lot of space in the final SVG given that such attributes are not
            // repeated.
            writeln!(
                f,
                r#"<g stroke="{}" stroke-width="{}" fill="none">"#,
                settings.stroke, stroke_width
            )?;

            for l in lines {
                dump_polyline(f, l, settings.digits)?;
            }

            writeln!(f, "</g>")?;

            Ok(())
        },
    )
}

pub fn dump_triangles_svg(
    path: &str,
    triangles: &[IsoTriangle<XY>],
    settings: &SvgSettings,
) -> io::Result<()> {
    svg_prelude(
        path,
        settings,
        || triangles.iter().flat_map(|l| l.pts.iter().copied()),
        |f, stroke_width| {
            for orient in [Orientation::Top, Orientation::Left, Orientation::Right] {
                let fill = settings.fill_colors[orient as usize];

                writeln!(
                    f,
                    r#"<g stroke="{fill}" fill="{fill}" stroke-width="{}" >"#,
                    stroke_width,
                    fill = fill.unwrap_or("none"),
                )?;

                for t in triangles {
                    if t.orientation != orient {
                        continue;
                    }

                    // be sure to close the polyline otherwise glitches occur
                    dump_polyline(
                        f,
                        &[t.pts[0], t.pts[1], t.pts[2], t.pts[0]],
                        settings.digits,
                    )?;
                }

                writeln!(f, "</g>")?;
            }

            Ok(())
        },
    )
}

fn svg_prelude<Pts>(
    path: &str,
    settings: &SvgSettings,
    get_pts: impl Fn() -> Pts,
    render: impl Fn(&mut BufWriter<File>, f64) -> io::Result<()>,
) -> io::Result<()>
where
    Pts: IntoIterator<Item = XY>,
{
    let f = File::create(path)?;
    let mut f = BufWriter::new(f);

    let (mut minx, mut maxx) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut miny, mut maxy) = (f64::INFINITY, f64::NEG_INFINITY);

    for (x, y) in get_pts() {
        minx = f64::min(minx, x);
        maxx = f64::max(maxx, x);

        miny = f64::min(miny, y);
        maxy = f64::max(maxy, y);
    }

    if minx > maxx || miny > maxy {
        writeln!(
            f,
            r#"<?xml version="1.0" encoding="UTF-8"?>
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 0 0">
    </svg>"#
        )?;
        return Ok(());
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

    // adjust the requested stroke_width according to the scaling factor
    let sw = settings.stroke_width / f64::max(settings.width / width, settings.height / height);

    render(&mut f, sw)?;

    writeln!(f, "</svg>")?;

    Ok(())
}

fn dump_polyline(f: &mut impl Write, pts: &[XY], digits: usize) -> io::Result<()> {
    write!(f, r#"<polyline points=""#)?;
    for (x, y) in pts {
        write!(f, "{x:.digits$},{y:.digits$} ", digits = digits)?;
    }
    writeln!(f, r#"" />"#)?;
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
            fill_colors: [None; 3],
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

    pub fn with_stroke_width(mut self, w: f64) -> Self {
        self.stroke_width = w;
        self
    }

    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_fill_color(mut self, orientation: Orientation, fill: &'a str) -> Self {
        self.fill_colors[orientation as usize] = Some(fill);
        self
    }
}
