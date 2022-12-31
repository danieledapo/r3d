use geo::{sdf::*, v3, Vec3};
use ivo::*;
use sketch_utils::opener;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(
        &(cuboid(v3(20, 20, 20))
            - (cuboid(v3(100, 18, 18)))
            - (cuboid(v3(18, 100, 18)))
            - (cuboid(v3(18, 18, 20)) + (v3(0, 0, 1)))
            - (cuboid(Vec3::replicate(15.0)) + (v3(10, 10, 10)))),
    );

    let triangles = render_outlines(&scene);

    dump_outlines_svg(
        "hollow_cube.svg",
        &triangles,
        &SvgSettings::new(1920.0, 1080.0),
    )
    .expect("cannot save hollow_cube.svg");

    opener::open("hollow_cube.svg").expect("cannot open hollow_cube.svg");
}
