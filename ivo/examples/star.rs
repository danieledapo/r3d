use geo::{chrono, util::opener};
use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let height = 500;
    let half_base = 100;
    let side = height + half_base;

    chrono!("scene", {
        for z in 0..height {
            let n = (1.0 - f64::from(z) / f64::from(height)) * f64::from(half_base);
            let n = n.round() as i32;

            for dy in -n..=n {
                for dx in -n..=n {
                    scene.add(dx, dy, half_base + z);
                    scene.add(half_base + z, dy, dx);
                    scene.add(dx, half_base + z, dy);

                    scene.add(dx, dy, -(half_base + z));
                    scene.add(-(half_base + z), dy, dx);
                    scene.add(dx, -(half_base + z), dy);
                }
            }
        }

        scene.invert();

        let step = 20;
        for i in (-side + step..=side).step_by(step as usize) {
            scene.aabb((i, 0, 0), (3, side, side));
            scene.aabb((0, i, 0), (side, 3, side));
            scene.aabb((0, 0, i), (side, side, 3));
        }
    });

    let triangles = chrono!("rendering", render(&scene));

    chrono!(
        "svg output",
        dump_svg(
            "star.svg",
            &triangles,
            &SvgSettings::new(1920.0 * 2.0, 1080.0 * 2.0)
                .with_stroke_width(2.0)
                .with_padding(20.0),
        )
        .expect("cannot save star.svg")
    );

    opener::open("star.svg").expect("cannot open star.svg");
}
