use geo::{v3, Vec3};
use sketch_utils::opener;

use buzz::*;

pub fn main() -> opener::Result<()> {
    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), v3(0, 0, 1)),
        Material::lambertian(v3(1, 1, 1)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0, 0, 1), 1.0),
        Material::lambertian(v3(1, 1, 1)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0, 0, 5), 1.0),
        Material::light(v3(0.5, 0.5, 0.5)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(v3(3, 3, 3), v3(0, 0, 0.5), v3(0, 0, 1), 50.0);

    let img = render(
        &camera,
        &scene,
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 5,
            samples: 10,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("hello.ppm").expect("cannot save output image");

    opener::open("hello.ppm")
}
