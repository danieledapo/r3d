use geo::vec3::Vec3;

use buzz::camera::Camera;
use buzz::material::Material;
use buzz::sphere::Sphere;
use buzz::Scene;

fn main() {
    // try to avoid aliasing by shooting multiple slightly different rays per
    // pixel and average the colors.
    let num_samples = 10;
    // let num_samples = 100;

    let target = Vec3::new(0.0, 0.0, -1.0);
    let camera = Camera::look_at(
        Vec3::new(3.0, 3.0, 2.0),
        target,
        Vec3::new(0.0, 1.0, 0.0),
        20.0,
    )
    .with_focus(target, 2.0);

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
        // Sphere::new(
        //     Vec3::new(-1.0, 0.0, -1.0),
        //     0.5,
        //     Material::Metal {
        //         albedo: Vec3::new(0.8, 0.8, 0.8),
        //         fuzziness: 0.8,
        //     },
        // ),
        Sphere::new(
            Vec3::new(-1.0, 0.0, -1.0),
            0.5,
            Material::Dielectric {
                refraction_index: 1.5,
            },
        ),
    ]);

    let mut rng = rand::thread_rng();

    let img = buzz::render(&camera, &scene, (400, 200), num_samples, &mut rng);
    img.save("img.ppm").unwrap();
}
