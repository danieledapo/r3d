use std::{env, path::Path};

use geo::{mesh::load_mesh, v3, Vec3};
use sketch_utils::opener;

use buzz::*;

const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(0.8, 0.1, 0.1));
// const MESH_MATERIAL: Material = Material::dielectric(2.4);

pub fn main() -> opener::Result<()> {
    let camera = Camera::look_at(v3(0.0, -4.0, 0.0), v3(0, 0, 0), v3(0, 0, 1), 35.0);
    // let camera = Camera::look_at(
    //     v3(0, 0, 2),
    //     v3(0, 0, 0),
    //     v3(0, 1, 0),
    //     80.0,
    // );

    let mut objects = SceneObjects::new();

    let suzanne = load_mesh(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("data")
            .join("suzanne.stl"),
    )
    .expect("cannot load suzanne.stl");

    for t in suzanne.triangles() {
        objects.push(Facet::new(t, &MESH_MATERIAL, true));
    }

    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(-0.5, -6.0, 0.0), 0.5),
        Material::light(v3(0.5, 0.5, 0.5)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0, 0, 6), 0.5),
        Material::light(v3(0.2, 0.2, 0.2)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0, 100, 0), 90.0),
        Material::lambertian(v3(0.2, 0.3, 0.36)),
    ));

    // let environment = Environment::Color(v3(0.8, 0.9, 0.8));
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

    img.save("suzanne.ppm").expect("cannot save output image");

    opener::open("suzanne.ppm")
}
