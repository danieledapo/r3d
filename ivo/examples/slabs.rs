use ivo::*;
use sketch_utils::opener;

use rand::prelude::*;

pub fn main() {
    let mut scene = Scene::new();

    let mut rng = rand::thread_rng();

    let side = 400;

    for i in 0..150 {
        let x: i32 = rng.gen_range(-6..=6) * side / 10;
        let y: i32 = rng.gen_range(-6..=6) * side / 10;
        let z: i32 = rng.gen_range(-6..=6) * side / 10;

        let w = i32::min((side * 9 / 10 - x).abs(), rng.gen_range(1..=20) * side / 20);
        let h = i32::min((side * 9 / 10 - y).abs(), rng.gen_range(1..=20) * side / 20);
        let d = i32::min((side * 9 / 10 - z).abs(), rng.gen_range(1..=20) * side / 20);

        let thickness = rng.gen_range(1..=5) * 2;

        if i % 3 == 0 {
            scene.aabb((x + w / 2, y + h / 2, z), (w / 2, h / 2, thickness));
        } else if i % 3 == 1 {
            scene.aabb((x, y + h / 2, z + d / 2), (thickness, h / 2, d / 2));
        } else {
            scene.aabb((x + w / 2, y, z + d / 2), (w / 2, thickness, d / 2));
        }
    }

    let triangles = render_outlines(&scene);

    dump_outlines_svg(
        "slabs.svg",
        &triangles,
        &SvgSettings::new(2048.0, 2048.0)
            .with_padding(20.0)
            .with_stroke_width(1.5),
    )
    .expect("cannot save slabs.svg");

    opener::open("slabs.svg").expect("cannot open slabs.svg");
}
