use geo::{mat4::Mat4, util::opener, Vec3};

use buzz::{
    csg::{self, SignedDistanceFunction},
    *,
};

pub fn main() -> opener::Result<()> {
    let plane = SimpleObject::new(
        PlaneGeometry::new(Vec3::new(0.0, 0.0, -0.5), Vec3::new(0.0, 0.0, 1.0)),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );

    let light1 = SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, -1.0, 0.25).normalized() * 4.0, 0.25),
        Material::light(Vec3::new(0.3, 0.3, 0.3)),
    );
    let light2 = SimpleObject::new(
        SphereGeometry::new(Vec3::new(-1.0, 1.0, 0.0).normalized() * 4.0, 0.25),
        Material::light(Vec3::new(0.3, 0.3, 0.3)),
    );

    let rounded_cube = csg::Sphere::new(0.65).intersection(csg::Cube::new(Vec3::replicate(1.0)));
    let cylinder = csg::Cylinder::new(0.25, 1.1);
    let cylinder_a = cylinder.clone().transformed(Mat4::rotate(
        Vec3::new(1.0, 0.0, 0.0),
        90.0_f64.to_radians(),
    ));
    let cylinder_b = cylinder.clone().transformed(Mat4::rotate(
        Vec3::new(0.0, 0.0, 1.0),
        90.0_f64.to_radians(),
    ));

    let csg = SimpleObject::new(
        SdfGeometry::new(
            rounded_cube
                .difference(cylinder)
                .difference(cylinder_a)
                .difference(cylinder_b)
                .transformed(Mat4::rotate(
                    Vec3::new(0.0, 0.0, 1.0),
                    30.0_f64.to_radians(),
                )),
        ),
        Material::lambertian(Vec3::new(0.31, 0.46, 0.22)),
    );

    let mut objects = SceneObjects::new();
    objects.push(light1);
    objects.push(light2);
    objects.push(plane);
    objects.push(csg);

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(
        Vec3::new(-3.0, 0.0, 1.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
        35.0,
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
    img.save("csg.png").expect("cannot save output image");

    opener::open("csg.png")
}
