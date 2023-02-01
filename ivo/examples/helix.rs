use std::{env, f64::consts::TAU};

use geo::util::arange;
use sketch_utils::{opener, sketch_output_path};

use ivo::*;

use noise::{NoiseFn, Perlin};
use rand::prelude::*;

#[derive(Debug, PartialEq, Eq)]
enum Mode {
    Cone,
    Noise,
}

pub fn main() {
    let mut scene = Scene::new();

    let _fill = ["true", "1"].contains(
        &env::args()
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_lowercase()
            .as_str(),
    );
    let mode = if env::args().nth(2).unwrap_or_default().trim().to_lowercase() == "cone" {
        Mode::Cone
    } else {
        Mode::Noise
    };

    let fill = false;

    let mut rng = thread_rng();

    let freq = rng.gen_range(0.3..=5.0);
    let freq2 = rng.gen_range(0.3..=3.0);
    let freq3 = rng.gen_range(0.3..=3.0);

    let nseeds = rng.gen_range(1..=20);
    let radius = if mode == Mode::Noise {
        rng.gen_range(30.0..=100.0)
    } else {
        rng.gen_range(30.0..=100.0)
    };

    let noise = Perlin::new(rng.gen());

    let mut seeds = vec![];
    for i in 1..=nseeds {
        let a = match mode {
            Mode::Cone => TAU / f64::from(i),
            Mode::Noise => f64::from(i) / f64::from(nseeds) * TAU,
        };
        seeds.push(a);
    }

    let height = 200.0;
    for zz in arange(0.0, height, 0.01) {
        let z = zz.round() as i32;

        let zt = zz / height;
        let a = zt * TAU;
        let a = a * freq;

        for ba in &seeds {
            let a = ba + a;

            let x;
            let y;
            match mode {
                Mode::Cone => {
                    x = a.cos() * radius * zt;
                    y = a.sin() * radius * zt;
                }
                Mode::Noise => {
                    let r = noise.get([a.cos() / freq2, a.sin() / freq2, zt * freq3]);
                    x = a.cos() * radius * r * rng.gen_range(0.8..=1.0);
                    y = a.sin() * radius * r * rng.gen_range(0.8..=1.0);
                }
            }

            scene.add(x as i32, y as i32, -z);
        }
    }

    let path = sketch_output_path("helix.svg").unwrap();

    let a4_multiplier = 3.0;
    let settings = SvgSettings::new(744.0 * a4_multiplier, 1052.0 * a4_multiplier)
        .with_stroke_width(2.0)
        .with_padding(10.0)
        .with_fill_color(Orientation::Top, "#ff0000")
        .with_fill_color(Orientation::Left, "#00ff00")
        .with_fill_color(Orientation::Right, "#0000ff");

    let settings = if fill {
        settings.with_background("#050609")
    } else {
        settings
    };

    if fill {
        let triangles = render_triangles(&scene);
        dump_triangles_svg(&path, &triangles, &settings).unwrap();
    } else {
        let triangles = render_outlines(&scene);
        dump_outlines_svg(&path, &triangles, &settings).unwrap();
    }

    opener::open(&path).expect("cannot open helix.svg");
}
