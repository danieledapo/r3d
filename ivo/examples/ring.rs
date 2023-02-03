use geo::sdf;
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(&sdf::torus(20.0, 200.0));

    let triangles = render_outlines(&scene);

    dump_outlines_svg("ring.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save ring.svg");

    opener::open("ring.svg").expect("cannot open ring.svg");
}
