pub mod camera;
pub mod material;
pub mod sphere;

use image::{Rgb, RgbImage};
use rand::Rng;

use geo::ray::Ray;
use geo::vec3;
use geo::vec3::Vec3;

use camera::Camera;
use material::Material;
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
    ]);

    let mut img = RgbImage::new(400, 200);

    let (w, h) = img.dimensions();

    let mut rng = rand::thread_rng();

    for (x, y, pix) in img.enumerate_pixels_mut() {
        let mut c = (0..num_samples)
            .map(|_| {
                let r = camera.cast_ray((x + rng.gen_range(0, 2), y + rng.gen_range(0, 2)), (w, h));
                color(&scene, &r, 0, &mut rng)
            })
            .sum::<Vec3>()
            / f64::from(num_samples);

        // gamma correct pixels
        c.x = c.x.sqrt();
        c.y = c.y.sqrt();
        c.z = c.z.sqrt();

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

fn color(scene: &Scene, ray: &Ray, depth: u32, rng: &mut impl Rng) -> Vec3 {
    if let Some((s, t)) = scene.intersection(ray) {
        // intersections too close to the ray's origin are caused by floating
        // point errors, consider them as not intersections...
        if t > 0.0001 {
            if depth >= 50 {
                return Vec3::zero();
            }

            match s.material {
                Material::Lambertian { albedo } => {
                    let intersection = ray.point_at(t);

                    return albedo
                        * color(
                            scene,
                            &Ray::new(
                                intersection,
                                s.normal_at(intersection) + random_in_circle(rng),
                            ),
                            depth + 1,
                            rng,
                        );
                }
            }
        }
    }

    // background
    let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
    vec3::lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), t)
}

fn random_in_circle(rng: &mut impl Rng) -> Vec3 {
    loop {
        let x = rng.gen();
        let y = rng.gen();
        let z = rng.gen();

        let v = Vec3::new(x, y, z) * 2.0 - 1.0;

        if v.norm2() >= 1.0 {
            break v;
        }
    }
}
