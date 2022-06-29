use std::sync::Arc;

use geo::{
    mat4::Mat4,
    sdf::{self, Sdf},
    util::opener,
    Aabb, Vec3,
};

use l::*;

#[derive(Debug)]
struct Thing;

impl Sdf for Thing {
    fn bbox(&self) -> Aabb {
        Aabb::cuboid(Vec3::zero(), 6.0)
    }

    fn dist(&self, p: &Vec3) -> f64 {
        let p = *p;

        let mut d = p.norm() - 2.5;
        d = f64::max(d, -((p - Vec3::new(0.0, -1.5, 0.0)).norm() - 1.5));

        d
    }
}

pub fn main() -> opener::Result<()> {
    let zslicer = ZSlicer::sdf(
        0.01,
        sdf::Torus::new(0.1, 0.5).union(
            sdf::Torus::new(0.1, 0.3)
                .transformed(Mat4::rotate(
                    Vec3::new(1.0, 0.0, 0.0),
                    std::f64::consts::FRAC_PI_2,
                ))
                .translate(Vec3::new(-1.5, 0.0, 0.0)),
        ),
    )
    .with_quantization_step(0.01);

    let scene = Scene::new(vec![Arc::new(zslicer) as Arc<dyn Object>]);

    let camera = Camera::look_at(
        Vec3::new(0.0, 4.0, 0.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
    )
    .with_perspective_projection(60.0, 1.0, 0.01, 100.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.01,
            simplify_eps: 0.00,
        },
    );
    dump_svg("expe.svg", &paths, SvgSettings::new(1024.0, 1024.0)).expect("cannot save expe.svg");

    opener::open("expe.svg")
}
