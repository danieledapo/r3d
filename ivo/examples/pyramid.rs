use geo::{sdf::*, Vec3};
use ivo::*;
use sketch_utils::opener;

fn pyramid(scene: &mut Scene, half_base: i32, height: i32) {
    for z in 0..height {
        let n = (1.0 - f64::from(z) / f64::from(height)) * f64::from(half_base);
        let n = n.round() as i32;

        scene.aabb((0, 0, z), (n, n, 0));
    }
}

pub fn main() {
    let mut scene = Scene::new();

    pyramid(&mut scene, 60, 200);
    scene.invert();
    scene.sdf(&(sphere(50.0) + Vec3::new(0.0, 0.0, 100.0)));

    let triangles = render_outlines(&scene);

    dump_outlines_svg("pyramid.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save pyramid.svg");

    opener::open("pyramid.svg").expect("cannot open pyramid.svg");
}
