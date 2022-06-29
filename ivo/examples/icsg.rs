use geo::{
    mat4::Mat4,
    sdf::{self, Sdf},
    Vec3,
};
use sketch_utils::opener;

use ivo::{dump_svg, render, Scene, SvgSettings};

pub fn main() {
    let rounded_cube = sdf::Sphere::new(0.65).intersection(sdf::Cuboid::new(Vec3::replicate(1.0)));
    let cylinder = sdf::Cylinder::new(0.25, 1.1);
    let cylinder_a = cylinder.clone().transformed(Mat4::rotate(
        Vec3::new(1.0, 0.0, 0.0),
        90.0_f64.to_radians(),
    ));
    let cylinder_b = cylinder.clone().transformed(Mat4::rotate(
        Vec3::new(0.0, 0.0, 1.0),
        90.0_f64.to_radians(),
    ));

    let mut scene = Scene::new();
    scene.sdf(
        &rounded_cube
            .difference(cylinder)
            .difference(cylinder_a)
            .difference(cylinder_b)
            .transformed(Mat4::rotate(Vec3::new(0.0, 0.0, 1.0), 0.0_f64.to_radians()))
            .transformed(Mat4::scale(Vec3::replicate(100.0))),
    );

    let tris = render(&scene);

    dump_svg("icsv.svg", &tris, &SvgSettings::new(1920.0, 1080.0)).unwrap();
    opener::open("icsv.svg").unwrap();
}
