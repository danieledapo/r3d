#![allow(clippy::useless_let_if_seq)]

pub mod camera;
pub mod material;
pub mod sphere;

use image::{Rgb, RgbImage};
use rand::Rng;
use rayon::prelude::*;

use geo::bvh::Bvh;
use geo::ray::Ray;
use geo::vec3;
use geo::vec3::Vec3;

use camera::Camera;
use material::{dielectric_bounce, lambertian_bounce, metal_bounce, Material};
use sphere::Sphere;

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug, PartialEq, Clone)]
pub struct Scene {
    objects: Bvh<Sphere>,
    environment: Environment,
}

/// The `Environment` surrounding the objects in a `Scene`. All the rays that
/// don't hit any objects will hit the environment.
#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    /// The `Environment` is a simple RGB color where each channel is in [0, 1].
    Color(Vec3),

    /// The `Environment` is a simple linear gradient between two RGB colors.
    LinearGradient(Vec3, Vec3),
}

impl Scene {
    /// Create a new `Scene` with the given objects inside the given
    /// `Environment`.
    pub fn new(objects: impl IntoIterator<Item = Sphere>, environment: Environment) -> Self {
        Scene {
            objects: objects.into_iter().collect(),
            environment,
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection<'s>(&'s self, ray: &'s Ray) -> Option<(&'s Sphere, f64)> {
        self.objects
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.partial_cmp(t1).unwrap())
    }
}

/// Simple struct to hold rendering params together.
#[derive(Debug, PartialEq, Clone)]
pub struct RenderConfig {
    /// how many samples to take for each pixel to find out its color. This can
    /// help reduce aliasing since the final color is the average of all the
    /// samples.
    pub samples: u32,

    /// the maximum number of bounces a ray can do. An higher value will give
    /// better results in scenes with a lot of reflective objects.
    pub max_bounces: u32,

    /// width and height of the rendered image.
    pub width: u32,
    pub height: u32,
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions.
pub fn render(
    camera: &Camera,
    scene: &Scene,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> image::RgbImage {
    let mut img = RgbImage::new(config.width, config.height);

    for (x, y, pix) in img.enumerate_pixels_mut() {
        *pix = render_pixel((x, y), camera, scene, rng, config);
    }

    img
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions concurrently.
pub fn parallel_render(camera: &Camera, scene: &Scene, config: &RenderConfig) -> image::RgbImage {
    let mut img = RgbImage::new(config.width, config.height);

    img.par_chunks_mut(3)
        .zip((0_u32..config.width * config.height).into_par_iter())
        .for_each(|(pix, i)| {
            let x = i % config.width;
            let y = i / config.width;

            let rgb = render_pixel(
                (x as u32, y as u32),
                camera,
                scene,
                &mut rand::thread_rng(),
                config,
            );

            pix[0] = rgb.data[0];
            pix[1] = rgb.data[1];
            pix[2] = rgb.data[2];
        });

    img
}

/// Render a single pixel of an image from a `Scene` and `Camera`.
pub fn render_pixel(
    (x, y): (u32, u32),
    camera: &Camera,
    scene: &Scene,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Rgb<u8> {
    let mut c = (0..config.samples)
        .map(|_| {
            let r = camera.cast_ray((x, y), (config.width, config.height), rng);
            sample(&scene, &r, 0, rng, config)
        })
        .sum::<Vec3>()
        / f64::from(config.samples);

    // gamma correct pixels
    c.x = c.x.sqrt();
    c.y = c.y.sqrt();
    c.z = c.z.sqrt();

    Rgb {
        data: [
            (c.x * 255.0) as u8,
            (c.y * 255.0) as u8,
            (c.z * 255.0) as u8,
        ],
    }
}

fn sample(scene: &Scene, ray: &Ray, depth: u32, rng: &mut impl Rng, config: &RenderConfig) -> Vec3 {
    let mut sample_material = |material: &Material, intersection, n| match *material {
        Material::Lambertian { albedo } => {
            albedo
                * sample(
                    scene,
                    &lambertian_bounce(intersection, n, rng),
                    depth + 1,
                    rng,
                    config,
                )
        }
        Material::Metal { albedo, fuzziness } => {
            let r = metal_bounce(ray, intersection, n, fuzziness, rng);

            if r.dir.dot(&n) < 0.0 {
                return Vec3::zero();
            }

            albedo * sample(scene, &r, depth + 1, rng, config)
        }
        Material::Dielectric { refraction_index } => sample(
            scene,
            &dielectric_bounce(ray, intersection, n, refraction_index, rng),
            depth + 1,
            rng,
            config,
        ),
        Material::Light { emittance } => emittance,
    };

    match scene.intersection(ray) {
        Some(_) if depth >= config.max_bounces => Vec3::zero(),
        Some((s, t)) => {
            let intersection = ray.point_at(t);
            let n = s.normal_at(intersection);

            sample_material(&s.material, intersection, n)
        }

        None => sample_environment(scene, ray),
    }
}

fn sample_environment(scene: &Scene, ray: &Ray) -> Vec3 {
    let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);

    match scene.environment {
        Environment::Color(c) => c,
        Environment::LinearGradient(a, b) => vec3::lerp(a, b, t),
    }
}
