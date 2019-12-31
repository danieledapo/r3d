use geo::mat4::Mat4;
use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::csg::{Cube, Cylinder, SdfGeometry, SignedDistanceFunction, Sphere};
use buzz::material::Material;
use buzz::plane::PlaneGeometry;
use buzz::sphere::SphereGeometry;
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene, SimpleObject};

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

    let rounded_cube = Sphere::new(0.65).intersection(Cube::new(Vec3::replicate(1.0)));
    let cylinder = Cylinder::new(0.25, 1.1);
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

    let scene = Scene::new(
        vec![
            Box::new(light1) as Box<dyn Object>,
            Box::new(light2) as Box<dyn Object>,
            Box::new(plane) as Box<dyn Object>,
            Box::new(csg) as Box<dyn Object>,
        ],
        Environment::Color(Vec3::zero()),
    );

    let camera = Camera::look_at(
        Vec3::new(-3.0, 0.0, 1.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
        35.0,
    );

    let img = parallel_render(
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
    img.save("csg.png")?;

    opener::open("csg.png")
}
