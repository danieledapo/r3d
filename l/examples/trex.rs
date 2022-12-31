use std::{path::Path, sync::Arc};

use geo::{mesh::load_mesh, v3, Vec3};
use sketch_utils::opener;

use l::*;

pub fn main() -> opener::Result<()> {
    let mesh = load_mesh(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("t-rex-skull.stl"),
    )
    .expect("cannot load t-rex-skull.stl");

    let position = v3(-30.0, -10.0, 10.0);
    let target = Vec3::zero();

    let scene = Scene::new(
        mesh.triangles()
            .map(|f| {
                Arc::new({
                    let light = f64::max(0.0, f.normal().dot((position - target).normalized()));
                    let lines = f64::floor(20.0 + (1.0 - light) * 10.0) as u16;

                    Facet::new(f).with_hatching_lines(lines)
                }) as Arc<dyn Object>
            })
            .collect::<Vec<_>>(),
    );

    let camera = Camera::look_at(position, target, v3(0, 0, 1))
        .with_perspective_projection(60.0, 1.0, 0.01, 10000.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.001,
            simplify_eps: 0.01,
        },
    );
    dump_svg("trex.svg", &paths, SvgSettings::new(2048.0, 2048.0)).expect("cannot save trex.svg");

    opener::open("trex.svg")
}
