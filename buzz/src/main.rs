use image::{Rgb, RgbImage};
use rand::Rng;

use geo::ray::Ray;
use geo::sphere;
use geo::vec3;
use geo::vec3::Vec3;

type Sphere = (Vec3, f64);
type Scene = Vec<Sphere>;

fn main() {
    let eye = Vec3::zero();
    let viewport_low_left = Vec3::new(-2.0, -1.0, -1.0);
    let viewport_width = Vec3::new(4.0, 0.0, 0.0);
    let viewport_height = Vec3::new(0.0, 2.0, 0.0);

    // try to avoid aliasing by shooting multiple slightly different rays per
    // pixel and average the colors.
    let num_samples = 10;

    let scene = vec![
        (Vec3::new(0.0, 0.0, -1.0), 0.5),
        (Vec3::new(0.0, -100.5, -1.0), 100.0),
    ];

    let mut img = RgbImage::new(400, 200);

    let w = f64::from(img.width());
    let h = f64::from(img.height());

    let mut rng = rand::thread_rng();

    for (x, y, pix) in img.enumerate_pixels_mut() {
        let c = (0..num_samples)
            .map(|_| {
                // convert canvas space y to world y since they're inverted.
                let v = 1.0 - f64::from(y + rng.gen_range(0, 2)) / h;
                let u = f64::from(x + rng.gen_range(0, 2)) / w;

                let r = Ray::new(
                    eye,
                    (viewport_low_left + viewport_width * u + viewport_height * v) - eye,
                );
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
    if let Some(((sphere_c, _), t)) = scene_ray_intersection(scene, ray) {
        let n = sphere::normal(sphere_c, ray.point_at(t));

        return (n + 1.0) * 0.5;
    }

    let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);

    vec3::lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), t)
}

fn scene_ray_intersection(scene: &Scene, ray: &Ray) -> Option<(Sphere, f64)> {
    scene
        .iter()
        .flat_map(|(c, r)| sphere::ray_intersection(*c, *r, ray).map(|t| ((*c, *r), t)))
        .min_by(|(_, t0): &(Sphere, f64), (_, t1): &(Sphere, f64)| t0.partial_cmp(t1).unwrap())
}
