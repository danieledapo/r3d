use std::{ops::Add, sync::Arc};

use geo::{mat4::Mat4, sdf::*, v3, Axis, Vec3};

use sketch_utils::opener;

use l::*;

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

    objects.push(Arc::new(SdfSlicer::new(sdf, 300, Axis::Z, light_dir)) as Arc<dyn Object>);

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

    objects.push(Arc::new(SdfSlicer::new(sdf, 300, Axis::Y, light_dir)) as Arc<dyn Object>);
    objects.push(Arc::new(SdfSlicer::new(
        octahedron(15.0) + v3(20.0, -20.0, 0),
        100,
        Axis::Z,
        light_dir,
    )) as Arc<dyn Object>);

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

    let hash = |p: Vec3| {
        let p = (p * 0.3183099 + v3(0.11, 0.17, 0.13)).fract() * 13.0;
        (p.x * p.y * p.z * (p.x + p.y + p.x)).fract()
    };
    let sph = move |i: Vec3, f: Vec3, c: Vec3| {
        let r = 0.5 * hash(i + c).abs();
        sphere(r).dist(&(f - c))
    };

    let rot = Mat4::rotate(v3(0, 0, 1), f64::to_radians(30.0));

    let sdf = (capsule(v3(0, 0, -25), v3(0, 0, 25), 50.0).shell(1.0)
        - (cuboid(v3(200, 200, 100)) + v3(0, 0, 85))
        - (sphere(30.0) + v3(0, -50, 0)))
    .then(move |op, mut d| {
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

    objects.push(Arc::new(SdfSlicer::new(sdf, 300, Axis::Z, light_dir)) as Arc<dyn Object>);

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
            stroke: "black",
            background: Some("white"),
            digits: 3,
        },
    )
    .expect("cannot save sdf.svg");

    opener::open("sdf.svg")
}
