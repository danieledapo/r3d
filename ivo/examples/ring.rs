use geo::{sdf::Sdf, Aabb, Vec3};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let r1 = 200.0;
    let r2 = 120.0;
    let rr = r1 + r2;

    let bbox = Aabb::new(Vec3::new(-rr, -rr, -r1)).expanded(Vec3::new(rr, rr, r1));

    let sdf = Sdf::from_fn(bbox, move |p| {
        let q = Vec3::new(Vec3::new(p.x, p.y, 0.0).norm() - r2, p.z, 0.0);
        q.norm2() - r1
    });

    scene.sdf(&sdf);

    let triangles = render_outlines(&scene);

    dump_outlines_svg("ring.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save ring.svg");

    opener::open("ring.svg").expect("cannot open ring.svg");
}
