use std::io::{BufReader, Cursor};

use geo::mesh::obj;
use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::facet::Facet;
use buzz::material::Material;
use buzz::sphere::SphereGeometry;
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene, SimpleObject};

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

    let mut objects = mesh
        .triangles()
        .map(|t| Box::new(Facet::new(t, &MESH_MATERIAL, true)) as Box<dyn Object>)
        .collect::<Vec<_>>();

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
        // FIXME: theoretically speaking, transmute should not be necessary
        // because rustc should automatically figure out the proper lifetimes.
        // In fact I assume it does so in the following call given that I'm not
        // specifying any lifetimes. I think it's the "scope" of the evaluation
        // that throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
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
