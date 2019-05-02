use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::plane::Plane;
use buzz::sphere::Sphere;
use buzz::{render, Environment, Object, RenderConfig, Scene};

pub fn main() -> opener::Result<()> {
    let plane = Plane::new(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    let sphere = Sphere::new(
        Vec3::new(0.0, 0.0, 1.0),
        1.0,
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    let light = Sphere::new(
        Vec3::new(0.0, 0.0, 5.0),
        1.0,
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    );

    let scene: Vec<Box<dyn Object>> = vec![
        Box::new(plane),
        Box::new(sphere),
        Box::new(light)
    ];

    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        50.0,
    );

    let background = Vec3::zero();

    let img = render(
        &camera,
        &Scene::new(scene, Environment::Color(background)),
        &mut rand::thread_rng(),
        &RenderConfig {
            width: 960,
            height: 540,
            max_bounces: 10,
            samples: 50,
            direct_lighting: true,
        },
    );
    img.save("hello.png")?;

    opener::open("hello.png")
}
