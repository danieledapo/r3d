use std::{ops::Add, sync::Arc};

use geo::{
    mat4::Mat4, primitive::polyline::Polyline, ray::Ray, sdf::*, spatial_index::Shape, v3, Aabb,
    Axis, Vec3,
};

use marching_squares::Field;
use sketch_utils::opener;

use rayon::prelude::*;

use l::*;

#[derive(Debug)]
struct Sp {
    sdf: Sdf,
    divs: u16,
    light_dir: Vec3,
    axis: Axis,
}

impl Shape for Sp {
    type Intersection = f64;

    fn intersection(&self, ray: &geo::ray::Ray) -> Option<Self::Intersection> {
        self.sdf.ray_march(ray, 128)
    }

    fn bbox(&self) -> Aabb {
        self.sdf.bbox()
    }
}

impl Object for Sp {
    fn paths(&self) -> Vec<Polyline> {
        let bbox = self.bbox();

        (0..self.divs)
            .into_par_iter()
            .flat_map(|t| {
                let mut paths = vec![];
                let y = bbox.min()[self.axis]
                    + bbox.dimensions()[self.axis] * f64::from(t) / f64::from(self.divs - 1);

                let f = SdfField::new(&self.sdf, y, self.axis);

                // f.debug_save();

                let contours = marching_squares::march(&f, 0.0);
                for c in contours {
                    let points = c
                        .into_iter()
                        .map(|(x, y)| {
                            let p = f.to_3d(x, y);

                            let d = (bbox.center() - p).normalized();
                            let t = self.sdf.ray_march(&Ray::new(p, d), 10).unwrap_or(0.0);
                            p + d * t
                        })
                        .collect::<Polyline>();

                    let mut cur = Polyline::new();
                    for p in points.chop(0.01).iter() {
                        let n = self.sdf.normal_at(p);
                        let lt = n.dot(-self.light_dir);

                        let black = lt <= 0.5 + 0.5 * ((t % 4) as f64 / 3.0);
                        if black {
                            cur.push(p);
                        } else if !cur.is_empty() {
                            paths.push(cur);
                            cur = Polyline::new();
                        }
                    }

                    if !cur.is_empty() {
                        paths.push(cur);
                    }
                }

                paths
            })
            .collect()
    }
}

struct SdfField {
    data: Vec<f64>,
    y: f64,
    axis: Axis,
    width: usize,
    height: usize,
    bbox: Aabb,
}

impl marching_squares::Field for SdfField {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.width + x]
    }
}

impl SdfField {
    const RESOLUTION: f64 = 10.0;

    pub fn new(sdf: &Sdf, y: f64, axis: Axis) -> Self {
        let bbox = sdf.bbox();
        let dims = bbox.dimensions();

        let (w, h) = match axis {
            Axis::X => (dims.y, dims.z),
            Axis::Y => (dims.z, dims.x),
            Axis::Z => (dims.x, dims.y),
        };
        let w = (w.ceil() * Self::RESOLUTION) as usize;
        let h = (h.ceil() * Self::RESOLUTION) as usize;

        // +1 to include the last pixel, +2 to add a black border
        let (w, h) = (w + 1 + 2, h + 1 + 2);

        let mut res = Self {
            bbox: sdf.bbox(),
            data: vec![100.0; w * h],
            y,
            axis,
            width: w,
            height: h,
        };

        const EPS: f64 = 0.001;
        for j in 0..h {
            // for i in 0..w {
            //     res.data[j * w + i] = sdf.dist(&res.to_3d(i as f64, j as f64));
            // }
            // continue;

            let mut i = 0.0;
            while i < w as f64 {
                let d = sdf.dist(&res.to_3d(i, j as f64));
                if d <= EPS {
                    res.data[j * w + i as usize] = d;

                    if i > 0.0 && res.data[j * w + i as usize - 1] > EPS {
                        res.data[j * w + i as usize - 1] = sdf.dist(&res.to_3d(i - 1.0, j as f64));
                    }

                    if j > 0 && res.data[(j - 1) * w + i as usize] > EPS {
                        res.data[(j - 1) * w + i as usize] =
                            sdf.dist(&res.to_3d(i, j as f64 - 1.0));
                    }

                    i += 1.0;
                    continue;
                }

                if i > 0.0 && res.data[j * w + i as usize - 1] <= EPS {
                    res.data[j * w + i as usize] = d;
                }

                if j > 0 && res.data[(j - 1) * w + i as usize] <= EPS {
                    res.data[j * w + i as usize] = d;
                }

                i += (d * Self::RESOLUTION).floor().max(1.0);
            }
        }

        res
    }

    pub fn to_3d(&self, i: f64, j: f64) -> Vec3 {
        let mut p = self.bbox.min();
        p -= 1.0 / Self::RESOLUTION;
        p[self.axis] = self.y;

        p += match self.axis {
            Axis::X => v3(0, i, j),
            Axis::Y => v3(j, 0, i),
            Axis::Z => v3(i, j, 0),
        } / v3(Self::RESOLUTION, Self::RESOLUTION, Self::RESOLUTION);

        p
    }

    pub fn debug_save(&self) {
        use std::{
            fs::File,
            io::{BufWriter, Write},
        };

        let mut out = BufWriter::new(
            File::create(format!(
                "debug-{:08}.pbm",
                (1000000.0 + self.y * 100.0).round() as i64
            ))
            .unwrap(),
        );

        let (w, h) = self.dimensions();

        writeln!(out, "P1").unwrap();
        writeln!(out, "{} {}", w, h).unwrap();

        for y in 0..h {
            for x in 0..w {
                write!(out, "{}", if self.z_at(x, y) < 0.001 { 0 } else { 1 }).unwrap();
            }
            writeln!(out).unwrap();
        }
    }
}

pub fn glitch_sdf() {
    let mut objects = vec![];

    let position = v3(0, -200, 100);
    let target = v3(0, 0, 0);
    let light_dir = v3(1, 1, -1).normalized();

    let sdf = Sdf::from_fn(sphere(70.0).bbox(), |&p| {
        let d = p.norm() - 70.0;
        d + 10.0 * (p.x * 0.5).cos()
    })
    .pad_bbox(100.0);

    objects.push(Arc::new(Sp {
        sdf,
        divs: 300,
        axis: Axis::Z,
        light_dir,
    }) as Arc<dyn Object>);

    let scene = Scene::new(objects);

    let camera = Camera::look_at(position, target, v3(0, 0, 1))
        .with_perspective_projection(45.0, 1.0, 0.001, 10000.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.01,
            simplify_eps: 0.001,
        },
    );
    dump_svg(
        "glitch_sdf.svg",
        &paths,
        SvgSettings {
            width: 2048.0,
            height: 2048.0,
            stroke_width: 3.0,
            stroke: "white",
            background: Some("black"),
            digits: 3,
        },
    )
    .expect("cannot save glitch_sdf.svg");

    opener::open("glitch_sdf.svg").unwrap();
}

pub fn poke_sdf() {
    let mut objects = vec![];

    let position = v3(0, -180, 0);
    let target = v3(0, 0, 0);
    let light_dir = v3(1, 1, -1).normalized();

    let sdf = sphere(50.0).shell(5.0) - cuboid(v3(300, 300, 30)) - sphere(30.0).add(v3(30, -30, 0));

    objects.push(Arc::new(Sp {
        sdf,
        divs: 300,
        axis: Axis::Y,
        light_dir,
    }) as Arc<dyn Object>);
    objects.push(Arc::new(Sp {
        sdf: octahedron(15.0) + v3(20.0, -20.0, 0),
        divs: 100,
        axis: Axis::Z,
        light_dir,
    }) as Arc<dyn Object>);

    let scene = Scene::new(objects);

    let camera = Camera::look_at(position, target, v3(0, 0, 1))
        .with_perspective_projection(45.0, 1.0, 0.001, 10000.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.01,
            simplify_eps: 0.001,
        },
    );
    dump_svg(
        "poke_sdf.svg",
        &paths,
        SvgSettings {
            width: 2048.0,
            height: 2048.0,
            stroke_width: 1.0,
            stroke: "black",
            background: None,
            digits: 3,
        },
    )
    .expect("cannot save poke_sdf.svg");

    opener::open("poke_sdf.svg").unwrap();
}

pub fn main() -> opener::Result<()> {
    let mut objects = vec![];

    let position = v3(0, -200, 100);
    let target = v3(0, 0, 0);
    let light_dir = v3(1, 1, -1).normalized();

    let sdf = capsule(v3(0, 0, -25), v3(0, 0, 25), 50.0).shell(1.0)
        - (cuboid(v3(200, 200, 100)) + v3(0, 0, 85))
        - (sphere(30.0) + v3(0, -50, 0));
    let hash = |p: Vec3| {
        let p = (p * 0.3183099 + v3(0.11, 0.17, 0.13)).fract() * 13.0;
        (p.x * p.y * p.z * (p.x + p.y + p.x)).fract()
    };
    let sph = move |i: Vec3, f: Vec3, c: Vec3| {
        let r = 0.5 * hash(i + c).abs();
        sphere(r).dist(&(f - c))
    };

    let rot = Mat4::rotate(v3(0, 0, 1), f64::to_radians(30.0));

    sdf.then(move |op, mut d| {
        let octaves = 3;

        let mut op = *op;

        let mut s = 1.0;
        for _ in 0..octaves {
            let p = op.abs() / (50.0 * s);
            let i = p.floor();
            let f = p.fract();

            let noise = s
                * 40.0
                * f64::min(
                    f64::min(
                        f64::min(sph(i, f, v3(0, 0, 0)), sph(i, f, v3(0, 0, 1))),
                        f64::min(sph(i, f, v3(0, 1, 0)), sph(i, f, v3(0, 1, 1))),
                    ),
                    f64::min(
                        f64::min(sph(i, f, v3(1, 0, 0)), sph(i, f, v3(1, 0, 1))),
                        f64::min(sph(i, f, v3(1, 1, 0)), sph(i, f, v3(1, 1, 1))),
                    ),
                );

            let noise = smooth_and(noise, d - 1.0 * s, 30.0 * s);
            d = smooth_union(noise, d, 30.0 * s);

            s *= 0.5;
            op = op * &rot;
        }

        d
    })
    .pad_bbox(30.0);

    objects.push(Arc::new(Sp {
        sdf,
        divs: 300,
        axis: Axis::Y,
        light_dir,
    }) as Arc<dyn Object>);

    let scene = Scene::new(objects);

    let camera = Camera::look_at(position, target, v3(0, 0, 1))
        .with_perspective_projection(45.0, 1.0, 0.001, 10000.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.01,
            simplify_eps: 0.001,
        },
    );
    dump_svg(
        "sdf.svg",
        &paths,
        SvgSettings {
            // width: 793.7007874015749,
            // height: 944.8818897637797,
            width: 2048.0,
            height: 2048.0,
            stroke_width: 3.0,
            stroke: "white",
            background: Some("black"),
            digits: 3,
        },
    )
    .expect("cannot save sdf.svg");

    opener::open("sdf.svg")
}
