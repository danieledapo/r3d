use geo::{sdf::*, v3};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(
        &(sphere(50.0)
            - (cuboid(v3(100, 100, 200)) + (v3(0.0, 0.0, -105.0)))
            - (cuboid(v3(20, 200, 200)))
            - (cuboid(v3(200, 20, 200)))),
    );

    let triangles = render_outlines(&scene);

    dump_outlines_svg("sphere.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save sphere.svg");

    opener::open("sphere.svg").expect("cannot open sphere.svg");
}
