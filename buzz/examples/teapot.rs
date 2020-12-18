use std::io::{BufReader, Cursor};

use geo::{mesh::obj, Vec3};

use buzz::*;

const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(1.0, 1.0, 0.95));

pub fn main() -> opener::Result<()> {
    let camera = Camera::look_at(
        Vec3::new(2.0, 5.0, -6.0),
        Vec3::new(0.5, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        35.0,
    );

    let mesh = obj::load_obj(BufReader::new(Cursor::new(
        &include_bytes!("../../data/teapot.obj")[..],
    )))
    .expect("cannot load teapot obj mesh");

    let mut objects = SceneObjects::new();
    for t in mesh.triangles() {
        objects.push(Facet::new(t, &MESH_MATERIAL, true));
    }

    objects.push(Box::new(SimpleObject::new(
        SphereGeometry::new(Vec3::new(2.0, 5.0, -3.0), 0.5),
        Material::light(Vec3::new(0.35, 0.35, 0.35)),
    )));
    objects.push(Box::new(SimpleObject::new(
        SphereGeometry::new(Vec3::new(5.0, 5.0, -3.0), 0.5),
        Material::light(Vec3::new(0.2, 0.2, 0.2)),
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

    img.save("teapot.png").expect("cannot save output image");

    opener::open("teapot.png")
}
