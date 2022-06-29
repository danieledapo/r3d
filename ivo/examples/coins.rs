use sketch_utils::opener;

use ivo::*;

use rand::prelude::*;

fn thing(scene: &mut Scene, (cx, cy, cz): Voxel, h: i32, r: i32) {
    for y in -r..=r {
        for x in -r..=r {
            if f64::hypot(f64::from(x), f64::from(y)) - f64::from(r) < 0.0 {
                for z in 0..h {
                    scene.add(cx + x, cy + y, cz + z);
                }
            }
        }
    }
}

pub fn main() {
    let mut scene = Scene::new();

    let mut rng = rand::thread_rng();
    let step = 10_i32;

    for z in (0..=600).step_by(step as usize) {
        let (dx, dy) = (rng.gen_range(-3..=3) * 5, rng.gen_range(-3..=3) * 5);

        thing(&mut scene, (dx, dy, z), step, rng.gen_range(3..=6) * step);
    }

    let triangles = render(&scene);

    dump_svg("coins.svg", &triangles, &SvgSettings::new(1080.0, 1920.0))
        .expect("cannot save coins.svg");

    opener::open("coins.svg").expect("cannot open coins.svg");
}
