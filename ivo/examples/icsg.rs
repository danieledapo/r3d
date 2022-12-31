use geo::{mat4::Mat4, sdf::*, v3};
use sketch_utils::opener;

use ivo::{dump_outlines_svg, render_outlines, Scene, SvgSettings};

pub fn main() {
    let rounded_cube = sphere(0.65) & cuboid(v3(1, 1, 1));
    let cylinder = cylinder(0.25, 1.1);
    let cylinder_a = cylinder.clone() * Mat4::rotate(v3(1, 0, 0), 90.0_f64.to_radians());
    let cylinder_b = cylinder.clone() * Mat4::rotate(v3(0, 0, 1), 90.0_f64.to_radians());

    let mut scene = Scene::new();
    scene.sdf(
        &((rounded_cube - cylinder - cylinder_a - cylinder_b)
            * Mat4::rotate(v3(0, 0, 1), 0.0)
            * Mat4::scale(v3(100, 100, 100))),
    );

    let tris = render_outlines(&scene);

    dump_outlines_svg("icsv.svg", &tris, &SvgSettings::new(1920.0, 1080.0)).unwrap();
    opener::open("icsv.svg").unwrap();
}
