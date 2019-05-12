use rand::Rng;

use geo::util::opener;
use geo::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::SphereGeometry;
use buzz::{parallel_render, Environment, RenderConfig, Scene, SimpleObject};

const SKY_ENVIRONMENT: Environment =
    Environment::LinearGradient(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0));

pub fn main() -> opener::Result<()> {
    let mut objects = Vec::with_capacity(400);
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, -1000.0, 0.0), 1000.0),
        Material::lambertian(Vec3::new(0.5, 0.5, 0.5)),
    ));

    let mut rng = rand::thread_rng();
    for a in -11..11 {
        for b in -11..11 {
            let center = Vec3::new(
                f64::from(a) + 0.9 * rng.gen::<f64>(),
                0.2,
                f64::from(b) + 0.9 * rng.gen::<f64>(),
            );

            if (center - Vec3::new(4.0, 0.2, 0.0)).norm() > 0.9 {
                let mp = rng.gen::<f64>();
                let mat = if mp < 0.8 {
                    Material::lambertian(rng.gen::<Vec3>() * rng.gen::<Vec3>())
                } else if mp < 0.95 {
                    Material::metal((rng.gen::<Vec3>() + 1.0) * 0.5, 0.5 * rng.gen::<f64>())
                } else {
                    Material::dielectric(1.5)
                };

                objects.push(SimpleObject::new(SphereGeometry::new(center, 0.2), mat));
            }
        }
    }

    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(0.0, 1.0, 0.0), 1.0),
        Material::dielectric(1.5),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(-4.0, 1.0, 0.0), 1.0),
        Material::lambertian(Vec3::new(0.4, 0.2, 0.1)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(4.0, 1.0, 0.0), 1.0),
        Material::metal(Vec3::new(0.7, 0.6, 0.5), 0.0),
    ));

    let scene = Scene::new(objects, SKY_ENVIRONMENT);

    let target = Vec3::zero();
    let camera = Camera::look_at(
        Vec3::new(13.0, 2.0, 3.0),
        target,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
    )
    // .with_focus(target, 0.1)
    ;

    let img = parallel_render(
        &camera,
        &scene,
        &RenderConfig {
            width: 1200,
            height: 800,
            max_bounces: 50,
            samples: 50,
            direct_lighting: false,
            soft_shadows: false,
        },
    );
    img.save("ray-tracing-in-a-weekend-cover.png")?;

    opener::open("ray-tracing-in-a-weekend-cover.png")
}
