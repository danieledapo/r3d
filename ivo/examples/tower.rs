use sketch_utils::opener;

use rand::prelude::*;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let mut args = std::env::args().skip(1);
    let mut next_arg = |def| args.next().map(|l| l.parse().unwrap()).unwrap_or(def);
    let n = next_arg(10);

    let mut rng = rand::thread_rng();

    let mut bz = 2;

    for _ in 0..n {
        let x = rng.gen_range(2..=100);
        let y = rng.gen_range(2..=100);
        let z = rng.gen_range(2..=bz);

        let w;
        let h;
        if rng.gen_bool(0.5) {
            w = rng.gen_range(10..=30);
            h = 1;
        } else {
            h = rng.gen_range(10..=30);
            w = 1;
        }
        let d = rng.gen_range(4..=14);

        scene.zslab((x, y, z), (w, h, d));
        scene.invert();

        bz = bz.max(z + d);
    }

    let triangles = render(&scene);

    dump_svg(
        "tower.svg",
        &triangles,
        &SvgSettings::new(2048.0, 2048.0)
            .with_padding(10.0)
            .with_stroke_width(2.0),
    )
    .expect("cannot save tower.svg");

    opener::open("tower.svg").expect("cannot open tower.svg");
}
