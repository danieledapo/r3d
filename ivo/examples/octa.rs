use geo::{
    sdf::{self, Sdf},
    Vec3,
};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let half_base = 60;
    let height = 200;

    for s in [-1, 1] {
        for z in 0..height {
            let n = (1.0 - f64::from(z) / f64::from(height)) * f64::from(half_base);
            let n = n.round() as i32;

            scene.aabb((0, 0, z * s), (n, n, 0));
        }

        scene.invert();
        scene.sdf(&sdf::Sphere::new(50.0).translate(Vec3::new(0.0, 0.0, f64::from(s) * 100.0)));
        scene.sdf(&sdf::Sphere::new(60.0).translate(Vec3::new(0.0, 0.0, f64::from(s) * 40.0)));

        scene.invert();
    }

    let triangles = render(&scene);

    dump_svg("octa.svg", &triangles, &SvgSettings::new(1080.0, 1920.0))
        .expect("cannot save octa.svg");

    opener::open("octa.svg").expect("cannot open octa.svg");
}
