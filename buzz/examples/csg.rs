use geo::util::opener;
use geo::{Aabb, Vec3};

use buzz::camera::Camera;
use buzz::csg::{CsgGeometry, CsgOp};
use buzz::cube::CubeGeometry;
use buzz::cylinder::CylinderGeometry;
use buzz::material::Material;
use buzz::plane::PlaneGeometry;
use buzz::sphere::SphereGeometry;
use buzz::{parallel_render, Environment, Object, RenderConfig, Scene, SimpleObject};

pub fn main() -> opener::Result<()> {
    let plane = SimpleObject::new(
        PlaneGeometry::new(Vec3::zero(), Vec3::new(0.0, 0.0, 1.0)),
        Material::lambertian(Vec3::new(1.0, 1.0, 1.0)),
    );
    // let light = SimpleObject::new(
    //     SphereGeometry::new(Vec3::new(3.0, 5.0, 0.0), 1.0),
    //     Material::light(Vec3::new(0.5, 0.5, 0.5)),
    // );
    let light = SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 5.0, 3.0), 1.0),
        Material::light(Vec3::new(0.3, 0.3, 0.3)),
    );

    let csg1 = SimpleObject::new(
        CsgGeometry::new(
            SphereGeometry::new(Vec3::new(2.0, 0.0, 1.0), 0.5),
            SphereGeometry::new(Vec3::new(2.5, 0.0, 1.0), 0.5),
            CsgOp::Union,
        ),
        Material::lambertian(Vec3::new(1.0, 0.2, 0.2)),
    );

    let csg2 = SimpleObject::new(
        CsgGeometry::new(
            // CubeGeometry::new(Aabb::cube(Vec3::new(0.5, -5.0, 1.0), 1.8)),
            SphereGeometry::new(Vec3::new(0.5, -5.0, 1.0), 1.5),
            SphereGeometry::new(Vec3::new(0.5, -4.0, 1.0), 0.8),
            CsgOp::Difference,
        ),
        Material::lambertian(Vec3::new(0.1, 0.1, 0.9)),
    );

    let csg3 = SimpleObject::new(
        CsgGeometry::new(
            CubeGeometry::new(Aabb::cube(Vec3::new(0.0, 0.0, 1.0), 1.8)),
            SphereGeometry::new(Vec3::new(0.0, 0.0, 1.0), 1.0),
            CsgOp::Intersection,
        )
        .difference(CylinderGeometry::new(0.2, (-0.5, 2.5))),
        Material::lambertian(Vec3::new(0.31, 0.46, 0.22)),
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
        // In fact I assume it does so in the following call given that I'm not
        // specifying any lifetimes. I think it's the "scope" of the evaluation
        // that throws it off.
        unsafe { std::mem::transmute::<_, &Scene<Box<dyn Object>>>(&scene) },
        &RenderConfig {
            width: 1920,
            height: 1080,
            max_bounces: 20,
            samples: 50,
            direct_lighting: true,
            soft_shadows: true,
        },
    );
    img.save("csg.png")?;

    opener::open("csg.png")
}
