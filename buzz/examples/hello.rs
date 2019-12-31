use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::plane::PlaneGeometry;
use buzz::sphere::SphereGeometry;
use buzz::{render, Environment, Object, RenderConfig, Scene, SimpleObject};

pub fn main() -> opener::Result<()> {
    let plane = SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0)),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    let sphere = SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, 1.0), 1.0),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    let light = SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, 5.0), 1.0),
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
        // In fact I assume it does so in the following call given that I'm not
        // specifying any lifetimes. I think it's the "scope" of the evaluation
        // that throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 5,
            samples: 10,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("hello.png")?;

    opener::open("hello.png")
}
