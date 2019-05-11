use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::csg::{Csg, CsgGeometry, CsgOp};
use buzz::material::Material;
use buzz::plane::Plane;
use buzz::sphere::{Sphere, SphereGeometry};
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene};

pub fn main() -> opener::Result<()> {
    let plane = Plane::new(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, 1.0),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    let light = Sphere::new(
        Vec3::new(0.0, 0.0, 5.0),
        1.0,
        Material::light(Vec3::new(0.5, 0.5, 0.5)),
    );

    let csg1 = Csg::new(
        CsgGeometry::new(
            SphereGeometry {
                center: Vec3::new(-0.5, 0.0, 1.0),
                radius: 1.0,
            },
            SphereGeometry {
                center: Vec3::new(0.5, 0.0, 1.0),
                radius: 1.0,
            },
            CsgOp::Intersection,
        ),
        Material::lambertian(Vec3::new(0.2, 1.0, 0.1)),
    );

    let csg2 = Csg::new(
        CsgGeometry::new(
            SphereGeometry {
                center: Vec3::new(2.0, 0.0, 1.0),
                radius: 0.5,
            },
            SphereGeometry {
                center: Vec3::new(2.5, 0.0, 1.0),
                radius: 0.5,
            },
            CsgOp::Union,
        ),
        Material::lambertian(Vec3::new(1.0, 0.2, 0.2)),
    );

    let csg3 = Csg::new(
        CsgGeometry::new(
            SphereGeometry {
                center: Vec3::new(-2.5, 0.5, 1.0),
                radius: 0.8,
            },
            SphereGeometry {
                center: Vec3::new(-2.0, 0.5, 1.0),
                radius: 0.8,
            },
            CsgOp::Difference,
        ),
        Material::lambertian(Vec3::new(0.1, 0.1, 0.9)),
    );

    let scene = Scene::new(
        vec![
            Box::new(plane) as Box<dyn Object>,
            Box::new(light) as Box<dyn Object>,
            Box::new(csg1) as Box<dyn Object>,
            Box::new(csg2) as Box<dyn Object>,
            Box::new(csg3) as Box<dyn Object>,
        ],
        Environment::Color(Vec3::zero()),
    );

    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::new(0.0, 0.0, 0.5),
        Vec3::new(0.0, 0.0, 1.0),
        50.0,
    );

    let img = parallel_render(
        &camera,
        // FIXME: theoretically speaking, transmute should not be necessary
        // because rustc should automatically figure out the proper lifetimes.
        // It doesn't seem to be a problem with the trait definitions themselves
        // because if the scene is composed of Box<Plane> only it compiles
        // perfectly. I think it's the manual cast to Box<dyn Object> that
        // throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
        &RenderConfig {
            width: 960,
            height: 540,
            max_bounces: 20,
            samples: 20,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("csg.png")?;

    opener::open("csg.png")
}
