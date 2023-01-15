use geo::{v3, Vec3};
use sketch_utils::opener;

use buzz::*;

fn main() -> opener::Result<()> {
    let target = v3(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, v3(0, 1, 0), 60.0);

    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0.0, 0.0, -1.0), 0.5),
        Material::lambertian(v3(0.8, 0.3, 0.3)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0.0, -100.5, -1.0), 100.0),
        Material::lambertian(v3(0.8, 0.8, 0)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(1.5, 0.0, -1.0), 0.5),
        Material::light(v3(0.5, 0.5, 0.5)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(-0.5, 1.0, 1.0), 0.3),
        Material::light(v3(0.2, 0.2, 0.2)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let img = render(
        &camera,
        &scene,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 10,
            max_bounces: 5,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("lights.ppm").expect("cannot save output image");

    opener::open("lights.ppm")
}
