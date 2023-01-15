use std::path::Path;

use geo::{mesh::load_mesh, v3, Vec3};
use sketch_utils::opener;

use buzz::*;

const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(1.0, 1.0, 0.95));

pub fn main() -> opener::Result<()> {
    let camera = Camera::look_at(v3(2.0, 5.0, -6.0), v3(0.5, 1, 0), v3(0, 1, 0), 35.0);

    let mut objects = SceneObjects::new();

    let teapot = load_mesh(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("teapot.obj"),
    )
    .expect("cannot load teapot.obj");

    for t in teapot.triangles() {
        objects.push(Facet::new(t, &MESH_MATERIAL, true));
    }

    objects.push(Box::new(SimpleObject::new(
        SphereGeometry::new(v3(2.0, 5.0, -3.0), 0.5),
        Material::light(v3(0.35, 0.35, 0.35)),
    )));
    objects.push(Box::new(SimpleObject::new(
        SphereGeometry::new(v3(5.0, 5.0, -3.0), 0.5),
        Material::light(v3(0.2, 0.2, 0.2)),
    )));

    let environment = Environment::Color(Vec3::zero());

    let scene = Scene::new(objects, environment);

    let img = parallel_render(
        &camera,
        &scene,
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 5,
            samples: 25,
            direct_lighting: true,
            soft_shadows: true,
        },
    );

    img.save("teapot.ppm").expect("cannot save output image");

    opener::open("teapot.ppm")
}
