use sketch_utils::opener;

use ivo::*;

use rand::prelude::*;

pub fn main() {
    let mut scene = Scene::with_dimensions_hint(600, 600, 600);

    let mut rng = rand::thread_rng();

    for z in (-400..=0).step_by(4) {
        for y in (-500..=500_i32).step_by(5) {
            if f64::from(z) < -75.0 - f64::powi(0.05 * f64::from(y), 2) {
                continue;
            }

            for x in (-50..=50).step_by(5) {
                if rng.gen_bool(0.8) {
                    continue;
                }

                let h = rng.gen_range(10..=20) * 2;
                let w = h / 2;

                scene.yslab((x, y, z), (w, h, 2));
            }
        }
    }

    let triangles = render(&scene);

    dump_svg("bridge.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save bridge.svg");

    opener::open("bridge.svg").expect("cannot open bridge.svg");
}
