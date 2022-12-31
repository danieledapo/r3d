use geo::{v3, Aabb, Vec3};
use sketch_utils::opener;

use ivo::*;

use rand::prelude::*;

pub fn main() {
    let mut scene = Scene::new();

    let mut rng = rand::thread_rng();

    let mut bboxes = vec![Aabb::cuboid(Vec3::zero(), 5.0)];
    for _ in 0..100_000 {
        let b = bboxes.choose(&mut rng).unwrap();

        let c = b.center().round();
        let d = b.dimensions().round();

        let t = rng.gen::<f64>();
        if t <= 0.3 {
            bboxes.push(Aabb::with_dimensions(
                c + v3(d.x * [-1.0, 1.0].choose(&mut rng).unwrap(), 0.0, 0.0),
                v3(
                    2.0 + 50.0 * rng.gen::<f64>(),
                    2.0 + 12.0 * rng.gen::<f64>(),
                    2.0 + 12.0 * rng.gen::<f64>(),
                ),
            ));
        } else if t <= 0.6 {
            bboxes.push(Aabb::with_dimensions(
                c + v3(0.0, d.y * [-1.0, 1.0].choose(&mut rng).unwrap(), 0.0),
                v3(
                    2.0 + 12.0 * rng.gen::<f64>(),
                    2.0 + 50.0 * rng.gen::<f64>(),
                    2.0 + 12.0 * rng.gen::<f64>(),
                ),
            ));
        } else {
            bboxes.push(Aabb::with_dimensions(
                c + v3(0.0, 0.0, d.z * [-1.0, 1.0].choose(&mut rng).unwrap()),
                v3(
                    2.0 + 12.0 * rng.gen::<f64>(),
                    2.0 + 12.0 * rng.gen::<f64>(),
                    2.0 + 50.0 * rng.gen::<f64>(),
                ),
            ));
        }
    }

    for (i, b) in bboxes.iter().enumerate() {
        let c = b.center().round();
        let d = b.dimensions();

        if (i % 7) > 0 {
            scene.invert();
        }
        scene.aabb(
            (c.x as i32, c.y as i32, c.z as i32),
            (d.x as i32, d.y as i32, d.z as i32),
        );
        if (i % 7) > 0 {
            scene.invert();
        }
    }

    let triangles = render_outlines(&scene);

    dump_outlines_svg(
        "archi.svg",
        &triangles,
        &SvgSettings::new(2048.0, 2048.0)
            .with_padding(20.0)
            .with_stroke_width(1.5),
    )
    .expect("cannot save archi.svg");

    opener::open("archi.svg").expect("cannot open archi.svg");
}
