use std::{path::Path, sync::Arc};

use geo::{mesh::load_mesh, util::opener, Vec3};

use l::*;

pub fn main() -> opener::Result<()> {
    let mesh = load_mesh(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("t-rex-skull.stl"),
    )
    .expect("cannot load t-rex-skull.stl");

    let scene = Scene::new(
        mesh.triangles()
            .map(|f| Arc::new(Facet::new(f)) as Arc<dyn Object>)
            .collect::<Vec<_>>(),
    );

    let camera = Camera::look_at(
        Vec3::new(-30.0, -10.0, 10.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
    )
    .with_perspective_projection(60.0, 1.0, 0.01, 10000.0);

    let paths = render(camera, &scene, &Settings { eps: 0.001 });
    dump_svg("trex.svg", &paths, (1024.0, 1024.0)).expect("cannot save trex.svg");

    opener::open("trex.svg")
}
