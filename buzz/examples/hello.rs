use geo::Vec3;
use sketch_utils::opener;

use buzz::*;

pub fn main() -> opener::Result<()> {
    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0)),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, 1.0), 1.0),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, 5.0), 1.0),
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        50.0,
    );

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
    img.save("hello.png").expect("cannot save output image");

    opener::open("hello.png")
}
