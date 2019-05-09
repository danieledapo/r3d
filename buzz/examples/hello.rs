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

    let scene = Scene::new(
        vec![
            Box::new(plane) as Box<dyn Object>,
            Box::new(sphere) as Box<dyn Object>,
            Box::new(light) as Box<dyn Object>,
        ],
        Environment::Color(Vec3::zero()),
    );

    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        50.0,
    );

    let img = render(
        &camera,
        // FIXME: theoretically speaking, transmute should not be necessary
        // because rustc should automatically figure out the proper lifetimes.
        // It doesn't seem to be a problem with the trait definitions themselves
        // because if the scene is composed of Box<Plane> only it compiles
        // perfectly. I think it's the manual cast to Box<dyn Object> that
        // throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
        &RenderConfig {
            width: 960,
            height: 540,
            max_bounces: 20,
            samples: 100,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("hello.png")?;

    opener::open("hello.png")
}
