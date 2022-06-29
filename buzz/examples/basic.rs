use geo::Vec3;
use sketch_utils::opener;

use buzz::*;

pub fn main() -> opener::Result<()> {
    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, Vec3::new(0.0, 1.0, 0.0), 90.0)
        .with_focus(target, 0.25);

    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 0.0, -1.0), 0.5),
        Material::lambertian(Vec3::new(0.8, 0.3, 0.3)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, -100.5, -1.0), 100.0),
        Material::lambertian(Vec3::new(0.8, 0.8, 0.0)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(1.0, 0.0, -1.0), 0.5),
        Material::metal(Vec3::new(0.8, 0.6, 0.2), 0.3),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(-1.0, 0.0, -1.0), 0.5),
        Material::dielectric(1.5),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::new(0.9, 0.9, 0.9)));

    let img = render(
        &camera,
        &scene,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 10,
            max_bounces: 5,
            direct_lighting: false,
            soft_shadows: false,
        },
    );
    img.save("basic.png").expect("cannot save output image");

    opener::open("basic.png")
}
