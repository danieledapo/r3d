use std::sync::Arc;

use geo::mat4::Mat4;
use geo::ray::Ray;
use geo::Axis;
use geo::{primitive::polyline::Polyline, sdf::*, spatial_index::Shape, v3, Aabb, Vec3};
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
        let t = self.sdf.ray_march(ray, 128)?;
        assert!(t.is_finite());
        Some(t)
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

                        let black = lt <= ((t % 4) as f64 / 3.0);
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

struct SdfField<'a> {
    data: &'a Sdf,
    y: f64,
    axis: Axis,
    width: usize,
    height: usize,
}

impl<'a> marching_squares::Field for SdfField<'a> {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data.dist(&self.to_3d(x as f64, y as f64))
    }
}

impl<'a> SdfField<'a> {
    const RESOLUTION: f64 = 10.0;

    pub fn new(sdf: &'a Sdf, y: f64, axis: Axis) -> Self {
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

        Self {
            data: sdf,
            y,
            axis,
            width: w,
            height: h,
        }
    }

    fn to_3d(&self, i: f64, j: f64) -> Vec3 {
        let mut p = self.data.bbox().min();
        p -= 1.0;

        p[self.axis] = self.y;

        p += match self.axis {
            Axis::X => v3(0, i, j),
            Axis::Y => v3(j, 0, i),
            Axis::Z => v3(i, j, 0),
        } / v3(Self::RESOLUTION, Self::RESOLUTION, Self::RESOLUTION);

        // are the points rounded correctly?
        p += 0.5;

        p
    }
}

pub fn main() -> opener::Result<()> {
    let mut objects = vec![];

    let position = v3(100, -150, 80);
    // let position = v3(0, -180, 0);
    let target = v3(0, 0, 0);
    let light_dir = v3(1, 0, 0).normalized();

    // let sdf = cuboid(v3(50, 50, 50));
    // let sdf = sphere(25.0) - sphere(15.0);
    // let sdf = sphere(25.0) - (sphere(15.0) + v3(0, -20, 0.0));
    // let sdf = sdf.shell(0.5) - (cuboid(v3(100, 100, 50)) + v3(0, 0, 30));
    // let sdf = (sphere(60.0) & cuboid(v3(100, 100, 100)))
    //     - capsule(v3(-100, 0, 0), v3(100, 0, 0), 15.0)
    //     - capsule(v3(0, -100, 0), v3(0, 100, 0), 15.0)
    //     - capsule(v3(0, 0, -100), v3(0, 0, 100), 15.0);
    // let sdf = (capsule(v3(0, 0, -25), v3(0, 0, 25), 50.0).shell(1.0)
    //     - (cuboid(v3(200, 200, 100)) + v3(0, 0, 85))
    //     - (sphere(30.0) + v3(0, -50, 0)))

    // let sdf = sphere(50.0).shell(5.0) - cuboid(v3(300, 300, 30)) - sphere(30.0).add(v3(30, -30, 0));

    let sdf = cylinder(25.0, 75.0).shell(10.0) - (cuboid(v3(100, 100, 100)) + v3(0, -75, 0));

    let hash = |p: Vec3| {
        let p = (p * 0.3183099 + v3(0.11, 0.17, 0.13)).fract() * 13.0;
        (p.x * p.y * p.z * (p.x + p.y + p.x)).fract()
    };
    let sph = move |i: Vec3, f: Vec3, c: Vec3| {
        let r = 0.5 * hash(i + c).abs();
        sphere(r).dist(&(f - c))
    };

    let rot = Mat4::rotate(v3(0, 0, 1), f64::to_radians(63.0));
    let sdf = sdf
        .then(move |op, mut d| {
            let mut op = *op;
            let octaves = 3;

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
        divs: 200,
        axis: Axis::Y,
        light_dir,
    }) as Arc<dyn Object>);

    // objects.push(Arc::new(Sp {
    //     sdf: octahedron(15.0) + v3(20.0, -20.0, 0),
    //     divs: 100,
    //     axis: Axis::Z,
    //     light_dir,
    // }) as Arc<dyn Object>);

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
            stroke_width: 1.0,
            stroke: "black",
            background: None,
            digits: 3,
        },
    )
    .expect("cannot save sdf.svg");

    opener::open("sdf.svg")
}
