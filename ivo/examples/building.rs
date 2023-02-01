use geo::{sdf::*, v3};
use sketch_utils::{opener, sketch_output_path};

use rand::prelude::*;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let path = sketch_output_path("building.svg").unwrap();

    scene.aabb((0, 0, 0), (100, 100, 100));

    scene.invert();

    let mut rng = thread_rng();
    for octave in 0..=6 {
        let n = 2 << octave;
        let r = 100 / (1 << octave);

        for _ in 0..n {
            scene.sdf(
                &(sphere(r.into())
                    + v3(
                        rng.gen_range(-100.0..=100.0),
                        rng.gen_range(-100.0..=100.0),
                        rng.gen_range(-100.0..=100.0),
                    )),
            );
        }
    }

    let settings = SvgSettings::new(744.0, 1052.0)
        .with_stroke_width(1.0)
        .with_padding(20.0);

    let triangles = render_outlines(&scene);
    dump_outlines_svg(&path, &triangles, &settings).unwrap();

    opener::open(&path).expect("cannot open building.svg");
}
