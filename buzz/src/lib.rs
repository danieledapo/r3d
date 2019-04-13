pub mod camera;
pub mod material;
pub mod sphere;
pub mod util;

use rand::Rng;
use image::{Rgb, RgbImage};

use geo::ray::Ray;
use geo::vec3;
use geo::vec3::Vec3;

use camera::Camera;
use material::Material;
use sphere::Sphere;
use util::random_in_unit_circle;

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    objects: Vec<Sphere>,
}

impl Scene {
    /// Create a new `Scene` with the given objects.
    pub fn new(objects: Vec<Sphere>) -> Self {
        Scene { objects }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection<'s>(&'s self, ray: &Ray) -> Option<(&'s Sphere, f64)> {
        self.objects
            .iter()
            .flat_map(|s| s.intersection(ray).map(|t| (s, t)))
            .min_by(|(_, t0), (_, t1)| t0.partial_cmp(t1).unwrap())
    }
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions. `num_samples` determines how many rays to cast per pixel to find
/// its color, the higher the better, but clearly it will be slower.
pub fn render(camera: &Camera, scene: &Scene, (width, height): (u32, u32), num_samples: u32, rng: &mut impl Rng) -> image::RgbImage {
    let mut img = RgbImage::new(width, height);

    for (x, y, pix) in img.enumerate_pixels_mut() {
        let mut c = (0..num_samples)
            .map(|_| {
                let r = camera.cast_ray((x, y), (width, height), rng);
                color(&scene, &r, 0, rng)
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

    img
}


fn color(scene: &Scene, ray: &Ray, depth: u32, rng: &mut impl Rng) -> Vec3 {
    if let Some((s, t)) = scene.intersection(ray) {
        // intersections too close to the ray's origin are caused by floating
        // point errors, consider them as not intersections...
        // TODO: instead of this, consider slightly changing the ray's position
        // towards its direction.
        if t > 0.001 {
            if depth >= 50 {
                return Vec3::zero();
            }

            let intersection = ray.point_at(t);
            let n = s.normal_at(intersection);

            match s.material {
                Material::Lambertian { albedo } => {
                    let r = Ray::new(intersection, n + random_in_unit_circle(rng));
                    return albedo * color(scene, &r, depth + 1, rng);
                }
                Material::Metal { albedo, fuzziness } => {
                    let r = Ray::new(
                        intersection,
                        Ray::new(ray.dir.normalized(), n).reflect()
                            + random_in_unit_circle(rng) * fuzziness,
                    );

                    if r.dir.dot(&n) < 0.0 {
                        return Vec3::zero();
                    }

                    return albedo * color(scene, &r, depth + 1, rng);
                }
                Material::Dielectric { refraction_index } => {
                    let outward_normal;
                    let ref_ix;
                    let cos;

                    if ray.dir.dot(&n) > 0.0 {
                        outward_normal = -n;
                        ref_ix = refraction_index;

                        // cos = ref_ix * ray.dir.dot(&n) / ray.dir.norm();
                        cos = (1.0
                            - ref_ix.powi(2) * (1.0 - (ray.dir.dot(&n) / ray.dir.norm()).powi(2)))
                        .sqrt();
                    } else {
                        outward_normal = n;
                        ref_ix = 1.0 / refraction_index;
                        cos = -ray.dir.dot(&n) / ray.dir.norm();
                    }

                    let dir = match Ray::new(ray.dir, outward_normal).refract(ref_ix) {
                        Some(refracted) => {
                            let reflect_prob = schlick(cos, ref_ix);

                            if rng.gen::<f64>() < reflect_prob {
                                Ray::new(ray.dir, n).reflect()
                            } else {
                                refracted
                            }
                        }
                        None => Ray::new(ray.dir, n).reflect(),
                    };

                    return color(scene, &Ray::new(intersection, dir), depth + 1, rng);
                }
            }
        }
    }

    // background
    let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
    vec3::lerp(Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.5, 0.7, 1.0), t)
}

/// Approximate the [Fresnel factor][1] that is the factor or refracted light
/// between different optical media using [Schlick equations].
///
/// [0]: https://en.wikipedia.org/wiki/Schlick's_approximation
/// [1]: https://en.wikipedia.org/wiki/Fresnel_equations
fn schlick(cos: f64, refraction_index: f64) -> f64 {
    let r0 = (1.0 - refraction_index) / (1.0 + refraction_index).powi(2);

    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}
