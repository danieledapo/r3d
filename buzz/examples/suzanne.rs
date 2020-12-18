use std::io::{BufReader, Cursor};

use geo::{mesh::stl, Vec3};

use buzz::*;

const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(0.8, 0.1, 0.1));
// const MESH_MATERIAL: Material = Material::dielectric(2.4);

pub fn main() -> opener::Result<()> {
    let camera = Camera::look_at(
        Vec3::new(0.0, -4.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        35.0,
    );
    // let camera = Camera::look_at(
    //     Vec3::new(0.0, 0.0, 2.0),
    //     Vec3::new(0.0, 0.0, 0.0),
    //     Vec3::new(0.0, 1.0, 0.0),
    //     80.0,
    // );

    let (_, tris) = stl::load_binary_stl(BufReader::new(Cursor::new(
        &include_bytes!("../../data/suzanne.stl")[..],
    )))?;

    let mut objects = SceneObjects::new();
    for t in tris {
        let t = t?;
        objects.push(Facet::new(t, &MESH_MATERIAL, true));
    }

    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(-0.5, -6.0, 0.0), 0.5),
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, 6.0), 0.5),
        Material::light(Vec3::new(0.2, 0.2, 0.2)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 100.0, 0.0), 90.0),
        Material::lambertian(Vec3::new(0.2, 0.3, 0.36)),
    ));

    // let environment = Environment::Color(Vec3::new(0.8, 0.9, 0.8));
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

    img.save("suzanne.png").expect("cannot save output image");

    opener::open("suzanne.png")
}
