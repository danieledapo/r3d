use std::sync::Arc;

use geo::{util::opener, Vec3};

use l::*;

pub fn main() -> opener::Result<()> {
    let fun = Grid::from_fn((-3.0, -3.0), (3.0, 3.0), 100, |x, y| {
        let x2 = x * x;
        let y2 = y * y;

        2.0 * (x2 + y2).sin() / (x2 + y2 + 0.0001)
    });

    let scene = Scene::new(vec![Arc::new(fun) as Arc<dyn Object>]);

    let camera = Camera::look_at(
        Vec3::new(8.0, 8.0, 10.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
    )
    .with_perspective_projection(30.0, 1.0, 0.01, 100.0);

    let paths = render(
        camera,
        &scene,
        &Settings {
            chop_eps: 0.01,
            simplify_eps: 0.001,
        },
    );
    dump_svg("fun.svg", &paths, SvgSettings::new(1024.0, 1024.0)).expect("cannot save fun.svg");

    opener::open("fun.svg")
}
