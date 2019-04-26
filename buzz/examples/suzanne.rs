use std::io;
use std::io::{BufReader, Cursor};

use geo::mesh::stl;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::facet::Facet;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::Object;
use buzz::{parallel_render, Environment, RenderConfig, Scene};

const MESH_MATERIAL: Material = Material::Lambertian {
    albedo: Vec3::new(0.8, 0.1, 0.1),
};

pub fn main() -> io::Result<()> {
    let camera = Camera::look_at(
        Vec3::new(2.7, -2.7, 2.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        35.0,
    );

    let mut scene: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(
            Vec3::new(-1.5, -4.5, 0.3),
            1.0,
            Material::Light {
                emittance: Vec3::new(9.0, 9.0, 9.0),
            },
        )),
        Box::new(Sphere::new(
            Vec3::new(2.1, 4.5, 0.3),
            1.0,
            Material::Light {
                emittance: Vec3::new(9.0, 9.0, 9.0),
            },
        )),
    ];

    let (_, tris) = stl::load_binary_stl(BufReader::new(Cursor::new(
        &include_bytes!("../../data/suzanne.stl")[..],
    )))?;

    for t in tris {
        let t = t?;
        scene.push(Box::new(Facet::new(t, &MESH_MATERIAL)));
    }

    let environment = Environment::Color(Vec3::new(0.2, 0.3, 0.36));
    let img = parallel_render(
        &camera,
        &Scene::new(scene, environment),
        &RenderConfig {
            width: 1920 / 2,
            height: 1080 / 2,
            max_bounces: 50,
            samples: 50,
        },
    );

    img.save("suzanne.png")
}
