use std::sync::Arc;

use rand::prelude::*;

use geo::{util::opener, Aabb, Vec3};

use l::*;

pub fn main() -> opener::Result<()> {
    let mut objects = vec![];

    let mut rng = thread_rng();
    for z in -10..=10 {
        for y in -10..=10 {
            for x in -10..=10 {
                if rng.gen::<f64>() <= 0.9 {
                    continue;
                }

                objects.push(Arc::new(Cube::new(Aabb::cube(
                    Vec3::new(x.into(), y.into(), z.into()),
                    1.0,
                ))) as Arc<dyn Object>);
            }
        }
    }

    let scene = Scene::new(objects);

    let camera = Camera::look_at(
        Vec3::new(-25.0, -25.0, -25.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
    )
    .with_perspective_projection(60.0, 1.0, 0.01, 100.0);

    let paths = render(camera, &scene, &Settings { eps: 0.001 });
    dump_svg("cube.svg", &paths, SvgSettings::new(2048.0, 2048.0)).expect("cannot save cube.svg");

    opener::open("cube.svg")
}
