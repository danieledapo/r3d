use geo::{
    mat4::{Mat4, Transform},
    util::opener,
    Aabb, Vec3,
};

use buzz::*;

pub fn main() -> opener::Result<()> {
    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0)),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(5.0, 5.0, 1.0), 1.0),
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    ));
    objects.push(SimpleObject::new(
        CylinderGeometry::new(0.25, (0.5, 2.5)),
        Material::lambertian(Vec3::new(0.31, 0.46, 0.22)),
    ));
    objects.push(SimpleObject::new(
        TransformedGeometry::new(
            CylinderGeometry::new(0.25, (0.0, 2.5)),
            Mat4::rotate(Vec3::new(0.0, 1.0, 0.0), 90.0_f64.to_radians())
                .transform(&Mat4::translate(Vec3::new(1.0, 0.0, -1.25))),
        ),
        Material::lambertian(Vec3::new(0.31, 0.46, 0.22)),
    ));
    objects.push(SimpleObject::new(
        CubeGeometry::new(Aabb::cube(Vec3::zero(), 1.0)),
        Material::lambertian(Vec3::new(0.88, 0.1, 0.1)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(
        Vec3::new(0.0, 3.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, 1.0),
        50.0,
    );

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
    img.save("cylinders.png").expect("cannot save output image");

    opener::open("cylinders.png")
}
