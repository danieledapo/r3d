use geo::{sdf::*, Vec3};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(
        &(sphere(50.0)
            - (cuboid(Vec3::new(100.0, 100.0, 200.0)) + (Vec3::new(0.0, 0.0, -105.0)))
            - (cuboid(Vec3::new(20.0, 200.0, 200.0)))
            - (cuboid(Vec3::new(200.0, 20.0, 200.0)))),
    );

    let triangles = render_outlines(&scene);

    dump_outlines_svg("sphere.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save sphere.svg");

    opener::open("sphere.svg").expect("cannot open sphere.svg");
}
