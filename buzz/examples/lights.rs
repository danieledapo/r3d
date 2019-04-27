use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{render, Environment, RenderConfig, Scene};

fn main() {
    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, Vec3::new(0.0, 1.0, 0.0), 60.0);

    let scene = Scene::new(
        vec![
            Sphere::new(
                Vec3::new(0.0, 0.0, -1.0),
                0.5,
                Material::lambertian(Vec3::new(0.8, 0.3, 0.3)),
            ),
            Sphere::new(
                Vec3::new(1.5, 0.0, -1.0),
                0.5,
                Material::light(Vec3::new(7.0, 7.0, 7.0)),
            ),
            Sphere::new(
                Vec3::new(-0.5, 1.0, 1.0),
                0.3,
                Material::light(Vec3::new(0.5, 0.5, 0.5)),
            ),
        ],
        Environment::Color(Vec3::zero()),
    );

    let mut rng = rand::thread_rng();

    let img = render(
        &camera,
        &scene,
        &mut rng,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 100,
            max_bounces: 50,
        },
    );
    img.save("lights.png").unwrap();
}
