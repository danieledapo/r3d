use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{render, Environment, RenderConfig, Scene};

fn main() -> opener::Result<()> {
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
                Vec3::new(0.0, -100.5, -1.0),
                100.0,
                Material::lambertian(Vec3::new(0.8, 0.8, 0.0)),
            ),
            Sphere::new(
                Vec3::new(1.5, 0.0, -1.0),
                0.5,
                Material::light(Vec3::new(0.5, 0.5, 0.5)),
            ),
            Sphere::new(
                Vec3::new(-0.5, 1.0, 1.0),
                0.3,
                Material::light(Vec3::new(0.2, 0.2, 0.2)),
            ),
        ],
        Environment::Color(Vec3::zero()),
    );

    let img = render(
        &camera,
        &scene,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 100,
            max_bounces: 50,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("lights.png")?;

    opener::open("lights.png")
}
