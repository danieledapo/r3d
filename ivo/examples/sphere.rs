use geo::{
    sdf::{self, Sdf},
    Vec3,
};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(
        &sdf::Sphere::new(50.0)
            .difference(
                sdf::Cuboid::new(Vec3::new(100.0, 100.0, 200.0))
                    .translate(Vec3::new(0.0, 0.0, -105.0)),
            )
            .difference(sdf::Cuboid::new(Vec3::new(20.0, 200.0, 200.0)))
            .difference(sdf::Cuboid::new(Vec3::new(200.0, 20.0, 200.0))),
    );

    let triangles = render(&scene);

    dump_svg("sphere.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save sphere.svg");

    opener::open("sphere.svg").expect("cannot open sphere.svg");
}
