use std::env;

use rand::Rng;

use geo::vec3::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::{render, RenderConfig, Scene};

fn main() {
    let s = env::args().nth(1).unwrap_or_else(|| "debug".to_string());

    match &s[..] {
        "debug" => debug(),
        "cover" => cover(),
        s => println!("unknown scene {}", s),
    }
}

pub fn debug() {
    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(Vec3::zero(), target, Vec3::new(0.0, 1.0, 0.0), 90.0)
        .with_focus(target, 0.25);

    let scene = Scene::new(vec![
        Sphere::new(
            Vec3::new(0.0, 0.0, -1.0),
            0.5,
            Material::Lambertian {
                albedo: Vec3::new(0.8, 0.3, 0.3),
            },
        ),
        Sphere::new(
            Vec3::new(0.0, -100.5, -1.0),
            100.0,
            Material::Lambertian {
                albedo: Vec3::new(0.8, 0.8, 0.0),
            },
        ),
        Sphere::new(
            Vec3::new(1.0, 0.0, -1.0),
            0.5,
            Material::Metal {
                albedo: Vec3::new(0.8, 0.6, 0.2),
                fuzziness: 0.3,
            },
        ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            Material::Dielectric {
                refraction_index: 1.5,
            },
        ),
    ]);

    let mut rng = rand::thread_rng();

    let img = render(
        &camera,
        &scene,
        &mut rng,
        &RenderConfig {
            width: 400,
            height: 200,
            samples: 10,
            max_bounces: 50,
        },
    );
    img.save("img.ppm").unwrap();
}

pub fn cover() {
    let mut scene = Vec::with_capacity(400);
    scene.push(Sphere::new(
        Vec3::new(0.0, -1000.0, 0.0),
        1000.0,
        Material::Lambertian {
            albedo: Vec3::new(0.5, 0.5, 0.5),
        },
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
                    Material::Lambertian {
                        albedo: rng.gen::<Vec3>() * rng.gen::<Vec3>(),
                    }
                } else if mp < 0.95 {
                    Material::Metal {
                        albedo: (rng.gen::<Vec3>() + 1.0) * 0.5,
                        fuzziness: 0.5 * rng.gen::<f64>(),
                    }
                } else {
                    Material::Dielectric {
                        refraction_index: 1.5,
                    }
                };

                scene.push(Sphere::new(center, 0.2, mat));
            }
        }
    }

    scene.push(Sphere::new(
        Vec3::new(0.0, 1.0, 0.0),
        1.0,
        Material::Dielectric {
            refraction_index: 1.5,
        },
    ));
    scene.push(Sphere::new(
        Vec3::new(-4.0, 1.0, 0.0),
        1.0,
        Material::Lambertian {
            albedo: Vec3::new(0.4, 0.2, 0.1),
        },
    ));
    scene.push(Sphere::new(
        Vec3::new(4.0, 1.0, 0.0),
        1.0,
        Material::Metal {
            albedo: Vec3::new(0.7, 0.6, 0.5),
            fuzziness: 0.0,
        },
    ));

    let target = Vec3::zero();
    let camera = Camera::look_at(
        Vec3::new(13.0, 2.0, 3.0),
        target,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
    )
    .with_focus(target, 0.1);

    let img = render(
        &camera,
        &Scene::new(scene),
        &mut rng,
        &RenderConfig {
            width: 1200,
            height: 800,
            max_bounces: 50,
            samples: 10,
        },
    );
    img.save("cover.ppm").unwrap();
}
