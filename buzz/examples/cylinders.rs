use geo::{mat4::Mat4, v3, Aabb, Vec3};
use sketch_utils::opener;

use buzz::*;

pub fn main() -> opener::Result<()> {
    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), v3(0, 0, 1)),
        Material::lambertian(v3(1, 1, 1)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(5, 5, 1), 1.0),
        Material::light(v3(0.5, 0.5, 0.5)),
    ));
    objects.push(SimpleObject::new(
        CylinderGeometry::new(0.25, (0.5, 2.5)),
        Material::lambertian(v3(0.31, 0.46, 0.22)),
    ));
    objects.push(SimpleObject::new(
        TransformedGeometry::new(
            CylinderGeometry::new(0.25, (0.0, 2.5)),
            Mat4::rotate(v3(0, 1, 0), 90.0_f64.to_radians())
                * &Mat4::translate(v3(1.0, 0.0, -1.25)),
        ),
        Material::lambertian(v3(0.31, 0.46, 0.22)),
    ));
    objects.push(SimpleObject::new(
        CubeGeometry::new(Aabb::cuboid(Vec3::zero(), 1.0)),
        Material::lambertian(v3(0.88, 0.1, 0.1)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(v3(0, 3, 1), v3(0, 0, 1), v3(0, 0, 1), 50.0);

    let img = parallel_render(
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
    img.save("cylinders.ppm").expect("cannot save output image");

    opener::open("cylinders.ppm")
}
