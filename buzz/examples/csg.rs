use geo::{mat4::Mat4, sdf::*, v3, Vec3};
use sketch_utils::opener;

use buzz::*;

pub fn main() -> opener::Result<()> {
    let plane = SimpleObject::new(
        PlaneGeometry::new(v3(0.0, 0.0, -0.5), v3(0, 0, 1)),
        Material::lambertian(v3(1, 1, 1)),
    );

    let light1 = SimpleObject::new(
        SphereGeometry::new(v3(0.0, -1.0, 0.25).normalized() * 4.0, 0.25),
        Material::light(v3(0.3, 0.3, 0.3)),
    );
    let light2 = SimpleObject::new(
        SphereGeometry::new(v3(-1.0, 1.0, 0.0).normalized() * 4.0, 0.25),
        Material::light(v3(0.3, 0.3, 0.3)),
    );

    let rounded_cube = sphere(0.65) & cuboid(v3(1.0, 1.0, 1.0));
    let cylinder = cylinder(0.25, 1.1);
    let cylinder_a = cylinder.clone() * Mat4::rotate(v3(1, 0, 0), 90.0_f64.to_radians());
    let cylinder_b = cylinder.clone() * Mat4::rotate(v3(0, 0, 1), 90.0_f64.to_radians());

    let csg = SimpleObject::new(
        SdfGeometry::new(
            (rounded_cube - cylinder - cylinder_a - cylinder_b)
                * Mat4::rotate(v3(0, 0, 1), 30.0_f64.to_radians()),
        ),
        Material::lambertian(v3(0.31, 0.46, 0.22)),
    );

    let mut objects = SceneObjects::new();
    objects.push(light1);
    objects.push(light2);
    objects.push(plane);
    objects.push(csg);

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(v3(-3.0, 0.0, 1.0), Vec3::zero(), v3(0, 0, 1), 35.0);

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
    img.save("csg.ppm").expect("cannot save output image");

    opener::open("csg.ppm")
}
