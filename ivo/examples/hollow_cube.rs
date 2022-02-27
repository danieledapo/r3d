use geo::{
    sdf::{self, Sdf},
    util::opener,
    Vec3,
};
use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(
        &sdf::Cuboid::new(Vec3::new(20.0, 20.0, 20.0))
            .difference(sdf::Cuboid::new(Vec3::new(100.0, 18.0, 18.0)))
            .difference(sdf::Cuboid::new(Vec3::new(18.0, 100.0, 18.0)))
            .difference(
                sdf::Cuboid::new(Vec3::new(18.0, 18.0, 20.0)).translate(Vec3::new(0.0, 0.0, 1.0)),
            )
            .difference(
                sdf::Cuboid::new(Vec3::replicate(15.0)).translate(Vec3::new(10.0, 10.0, 10.0)),
            ),
    );

    let triangles = render(&scene);

    dump_svg(
        "hollow_cube.svg",
        &triangles,
        &SvgSettings::new(1920.0, 1080.0),
    )
    .expect("cannot save hollow_cube.svg");

    opener::open("hollow_cube.svg").expect("cannot open hollow_cube.svg");
}