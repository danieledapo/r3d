use std::f64::consts::TAU;

use sketch_utils::{opener, sketch_output_path};

use ivo::*;

use rand::prelude::*;

pub fn main() {
    let mut scene = Scene::new();

    let mut rng = rand::thread_rng();

    let n = rng.gen::<u8>();

    let mut cells: Vec<u8> = vec![0; 360];
    for _ in 0..rng.gen_range(1..=10) {
        let i = rng.gen_range(0..cells.len());
        cells[i] = 1;
    }

    for z in 0..100 {
        let mut new = vec![0; cells.len()];

        for (x, c) in cells.iter().enumerate() {
            let a = f64::from(x as i32) / (cells.len() as f64) * TAU;

            let xx = f64::round(a.cos() * 50.0) as i32;
            let yy = f64::round(a.sin() * 50.0) as i32;

            // if *c != 0 {
            // if *c != 0 && (xx > 0 || (yy & 1) != 0) {
            if *c != 0 && (xx & 1) != 0 {
                scene.add(xx, yy, -z);
            }

            if x > 0 && x < cells.len() - 1 {
                let ix = (cells[x - 1] << 2) | (c << 1) | (cells[x + 1]);
                new[x] = (n >> ix) & 1;
            }
        }

        cells = new;
    }

    let triangles = render_outlines(&scene);

    let path = sketch_output_path("automata.svg").unwrap();

    dump_outlines_svg(
        &path,
        &triangles,
        // &SvgSettings::new(744.0, 1052.0)
        &SvgSettings::new(744.0 * 3.0, 1052.0 * 3.0)
            .with_stroke_width(2.0)
            .with_padding(10.0),
    )
    .unwrap();

    opener::open(&path).expect("cannot open automata.svg");
}
