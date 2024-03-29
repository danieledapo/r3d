use rand::Rng;

use buzz::*;
use geo::{v3, Vec3};
use sketch_utils::opener;

const SKY_ENVIRONMENT: Environment =
    Environment::LinearGradient(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0));

pub fn main() -> opener::Result<()> {
    let mut objects = SceneObjects::new();
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(0.0, -1000.0, 0.0), 1000.0),
        Material::lambertian(v3(0.5, 0.5, 0.5)),
    ));

    let mut rng = rand::thread_rng();
    for a in -11..11 {
        for b in -11..11 {
            let center = v3(
                f64::from(a) + 0.9 * rng.gen::<f64>(),
                0.2,
                f64::from(b) + 0.9 * rng.gen::<f64>(),
            );

            if (center - v3(4, 0.2, 0)).norm() > 0.9 {
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
        SphereGeometry::new(v3(0, 1, 0), 1.0),
        Material::dielectric(1.5),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(-4.0, 1.0, 0.0), 1.0),
        Material::lambertian(v3(0.4, 0.2, 0.1)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(v3(4, 1, 0), 1.0),
        Material::metal(v3(0.7, 0.6, 0.5), 0.0),
    ));

    let scene = Scene::new(objects, SKY_ENVIRONMENT);

    let target = Vec3::zero();
    let camera = Camera::look_at(
        v3(13, 2, 3),
        target,
        v3(0, 1, 0),
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
    img.save("ray-tracing-in-a-weekend-cover.ppm")
        .expect("cannot save output image");

    opener::open("ray-tracing-in-a-weekend-cover.ppm")
}
