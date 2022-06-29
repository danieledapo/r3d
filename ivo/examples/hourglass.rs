use geo::{
    sdf::{self, Sdf},
    util::opener,
    Vec3,
};

use ivo::*;

pub fn main() {
    let mut scene = Scene::with_dimensions_hint(300, 300, 300);

    let n = 7;
    for i in 0..n {
        let t = f64::from(i) / f64::from(n - 1);

        let r = t * 80.0;
        let z = t * 160.0;

        scene.sdf(&sdf::Torus::new(3.0, r).translate(Vec3::new(0.0, 0.0, z)));
        scene.sdf(&sdf::Torus::new(3.0, r).translate(Vec3::new(0.0, 0.0, -z)));
    }

    let triangles = render(&scene);

    dump_svg(
        "hourglass.svg",
        &triangles,
        &SvgSettings::new(1080.0, 1920.0),
    )
    .expect("cannot save hourglass.svg");

    opener::open("hourglass.svg").expect("cannot open hourglass.svg");
}
