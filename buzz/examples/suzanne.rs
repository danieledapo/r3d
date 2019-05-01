use std::io;
use std::io::{BufReader, Cursor};

use geo::mesh::stl;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::facet::Facet;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene};

const MESH_MATERIAL: Material = Material::lambertian(Vec3::new(0.8, 0.1, 0.1));
// const MESH_MATERIAL: Material = Material::dielectric(2.4);

pub fn main() -> io::Result<()> {
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

    let mut scene: Vec<Box<dyn Object>> = tris
        .into_iter()
        .map(|t| {
            let t = t?;
            Ok(Box::new(Facet::new(t, &MESH_MATERIAL, true)) as Box<dyn Object>)
        })
        .collect::<io::Result<Vec<_>>>()?;

    scene.push(Box::new(Sphere::new(
        Vec3::new(-0.5, -6.0, 0.0),
        0.5,
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    )));
    scene.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, 6.0),
        0.5,
        Material::light(Vec3::new(0.2, 0.2, 0.2)),
    )));
    scene.push(Box::new(Sphere::new(
        Vec3::new(0.0, 100.0, 0.0),
        90.0,
        Material::lambertian(Vec3::new(0.2, 0.3, 0.36)),
    )));

    // let environment = Environment::Color(Vec3::new(0.8, 0.9, 0.8));
    let environment = Environment::Color(Vec3::zero());

    let img = parallel_render(
        &camera,
        &Scene::new(scene, environment),
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 50,
            samples: 25,
            direct_lighting: true,
        },
    );

    img.save("suzanne.png")
}
