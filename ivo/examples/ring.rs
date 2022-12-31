use geo::{sdf::Sdf, v3, Aabb};
use sketch_utils::opener;

use ivo::*;

pub fn main() {
    let mut scene = Scene::new();

    let r1 = 200.0;
    let r2 = 120.0;
    let rr = r1 + r2;

    let bbox = Aabb::new(v3(-rr, -rr, -r1)).expanded(v3(rr, rr, r1));

    let sdf = Sdf::from_fn(bbox, move |p| {
        let q = v3(v3(p.x, p.y, 0.0).norm() - r2, p.z, 0.0);
        q.norm2() - r1
    });

    scene.sdf(&sdf);

    let triangles = render_outlines(&scene);

    dump_outlines_svg("ring.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save ring.svg");

    opener::open("ring.svg").expect("cannot open ring.svg");
}
