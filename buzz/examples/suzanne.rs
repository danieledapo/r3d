use std::io;
use std::io::{BufReader, Cursor};

use geo::mesh::stl;
use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::facet::Facet;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene};

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

    let mut objects = tris
        .into_iter()
        .map(|t| {
            let t = t?;
            Ok(Box::new(Facet::new(t, &MESH_MATERIAL, true)) as Box<dyn Object>)
        })
        .collect::<io::Result<Vec<_>>>()?;

    objects.push(Box::new(Sphere::new(
        Vec3::new(-0.5, -6.0, 0.0),
        0.5,
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    )));
    objects.push(Box::new(Sphere::new(
        Vec3::new(0.0, 0.0, 6.0),
        0.5,
        Material::light(Vec3::new(0.2, 0.2, 0.2)),
    )));
    objects.push(Box::new(Sphere::new(
        Vec3::new(0.0, 100.0, 0.0),
        90.0,
        Material::lambertian(Vec3::new(0.2, 0.3, 0.36)),
    )));

    // let environment = Environment::Color(Vec3::new(0.8, 0.9, 0.8));
    let environment = Environment::Color(Vec3::zero());

    let scene = Scene::new(objects, environment);

    let img = parallel_render(
        &camera,
        // FIXME: theoretically speaking, transmute should not be necessary
        // because rustc should automatically figure out the proper lifetimes.
        // It doesn't seem to be a problem with the trait definitions themselves
        // because if the scene is composed of Box<Plane> only it compiles
        // perfectly. I think it's the manual cast to Box<dyn Object> that
        // throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 50,
            samples: 25,
            direct_lighting: true,
            soft_shadows: true,
        },
    );

    img.save("suzanne.png")?;

    opener::open("suzanne.png")
}
