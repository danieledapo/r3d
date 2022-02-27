use geo::{
    sdf::{self, Sdf},
    util::opener,
    Vec3,
};
use ivo::*;

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
    scene.sdf(&sdf::Sphere::new(50.0).translate(Vec3::new(0.0, 0.0, 100.0)));

    let triangles = render(&scene);

    dump_svg("pyramid.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save pyramid.svg");

    opener::open("pyramid.svg").expect("cannot open pyramid.svg");
}
