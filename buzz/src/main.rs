pub mod camera;
pub mod sphere;

use image::{Rgb, RgbImage};
use rand::Rng;

use geo::ray::Ray;
use geo::vec3;
use geo::vec3::Vec3;

use camera::Camera;
use sphere::Sphere;

#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    objects: Vec<Sphere>,
}

impl Scene {
    pub fn new(objects: Vec<Sphere>) -> Self {
        Scene { objects }
    }

    pub fn intersection<'s>(&'s self, ray: &Ray) -> Option<(&'s Sphere, f64)> {
        self.objects
            .iter()
            .flat_map(|s| s.intersection(ray).map(|t| (s, t)))
            .min_by(|(_, t0), (_, t1)| t0.partial_cmp(t1).unwrap())
    }
}

fn main() {
    // try to avoid aliasing by shooting multiple slightly different rays per
    // pixel and average the colors.
    let num_samples = 10;

    let camera = Camera::look_at(
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
    );

    let scene = Scene::new(vec![
        Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5),
        Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0),
    ]);

    let mut img = RgbImage::new(400, 200);

    let (w, h) = img.dimensions();

    let mut rng = rand::thread_rng();

    for (x, y, pix) in img.enumerate_pixels_mut() {
        let c = (0..num_samples)
            .map(|_| {
                let r = camera.cast_ray((x + rng.gen_range(0, 2), y + rng.gen_range(0, 2)), (w, h));
                color(&scene, &r)
            })
            .sum::<Vec3>()
            / f64::from(num_samples);

        *pix = Rgb {
            data: [
                (c.x * 255.0) as u8,
                (c.y * 255.0) as u8,
                (c.z * 255.0) as u8,
            ],
        };
    }

    img.save("img.ppm").unwrap();
}

fn color(scene: &Scene, ray: &Ray) -> Vec3 {
    if let Some((s, t)) = scene.intersection(ray) {
        let n = s.normal_at(ray.point_at(t));

        return (n + 1.0) * 0.5;
    }

    let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);

    vec3::lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), t)
}
