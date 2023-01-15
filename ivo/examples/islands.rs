use std::collections::HashMap;

use noise::{HybridMulti, MultiFractal, NoiseFn, Perlin};
use rand::prelude::*;

use ivo::*;
use sketch_utils::{opener, sketch_output_path};

fn landscape(scene: &mut Scene, noise: impl NoiseFn<f64, 2>) {
    let mut minh = f64::INFINITY;
    let mut maxh = f64::NEG_INFINITY;

    let mut values = HashMap::new();
    let mut stack = vec![];

    for y in -20_i32..=20 {
        for x in -20_i32..=20 {
            stack.push((x, y));
        }
    }

    while let Some((x, y)) = stack.pop() {
        use std::collections::hash_map::Entry;

        let v = match values.entry((x, y)) {
            Entry::Occupied(_) => continue,
            Entry::Vacant(v) => v,
        };

        let n = noise.get([f64::from(x) / 100.0, f64::from(y) / 100.0]);
        v.insert(n);

        minh = minh.min(n);
        maxh = maxh.max(n);

        if n > 0.15 {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if !values.contains_key(&(x + dx, y + dy)) {
                        stack.push((x + dx, y + dy));
                    }
                }
            }
        }
    }

    for ((x, y), h) in values {
        // let h = (h - minh) / (maxh - minh);
        let h = h * 100.0;
        let h = (h / 10.0).round() * 10.0;

        scene.zslab((x, y, 0), (0, 0, h as i32));
    }
}

pub fn main() {
    let mut scene = Scene::new();

    let seed = rand::thread_rng().gen();

    let noise = HybridMulti::<Perlin>::new(seed)
        .set_frequency(0.5)
        .set_octaves(2);

    landscape(&mut scene, noise);

    let triangles = render_outlines(&scene);

    let path = sketch_output_path("islands.svg").unwrap();

    dump_outlines_svg(
        &path,
        &triangles,
        // &SvgSettings::new(2048.0, 2048.0)
        &SvgSettings::new(1052.0, 744.0)
            .with_padding(10.0)
            .with_stroke_width(1.0),
    )
    .expect("cannot save output image");

    opener::open(path).expect("cannot open output image");
}
