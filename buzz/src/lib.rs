#![allow(clippy::useless_let_if_seq)]

pub mod camera;
pub mod facet;
pub mod material;
pub mod plane;
pub mod sphere;

use std::ops::Deref;

use image::{Rgb, RgbImage};
use rand::prelude::*;
use rayon::prelude::*;

use geo::ray::Ray;
use geo::spatial_index::Bvh;
use geo::spatial_index::Shape;
use geo::{vec3, Vec3};

use camera::Camera;
use material::{dielectric_bounce, lambertian_bounce, metal_bounce, Material};

/// An `Object` that can be rendered.
pub trait Object: Shape + Sync {
    /// Getter for the `Material` the `Object` is made of.
    fn material(&self) -> &Material;

    /// Calculate the normal for the given point `p`. This method should never
    /// be called if the `Object` does not intersect it.
    fn normal_at(&self, p: Vec3) -> Vec3;

    /// Get a bounding sphere for this `Object`. This is used in case the
    /// `Object` is used with a `Light` material.
    fn bounding_sphere(&self) -> (Vec3, f64);
}

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug)]
pub struct Scene<O: Object> {
    objects: Bvh<O>,
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

impl<O: Object> Scene<O> {
    /// Create a new `Scene` with the given objects inside the given
    /// `Environment`.
    pub fn new(objects: impl IntoIterator<Item = O>, environment: Environment) -> Self {
        Scene {
            objects: objects.into_iter().collect(),
            environment,
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection<'s>(&'s self, ray: &'s Ray) -> Option<(&'s O, f64)> {
        self.objects
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.partial_cmp(t1).unwrap())
    }

    /// Return an iterator over all the lights in the `Scene`.
    pub fn lights(&self) -> impl Iterator<Item = &O> {
        self.objects.iter().filter(|o| {
            if let Material::Light { .. } = o.material() {
                true
            } else {
                false
            }
        })
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

    /// whether to calculate direct lighting for each intersection. This is
    /// useful because calculating only indirect lighting in a scene is
    /// particularly resource hungry if a lot of details is needed. On the other
    /// hand, mother nature only uses indirect lighting and therefore direct
    /// lighting feels a bit "artificial".
    pub direct_lighting: bool,

    /// try to smooth shadows a bit to make them a bit more gradual.
    pub soft_shadows: bool,

    /// width and height of the rendered image.
    pub width: u32,
    pub height: u32,
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions.
pub fn render(
    camera: &Camera,
    scene: &Scene<impl Object>,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> image::RgbImage {
    let lights = if config.direct_lighting {
        scene.lights().collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut img = RgbImage::new(config.width, config.height);

    for (x, y, pix) in img.enumerate_pixels_mut() {
        *pix = render_pixel((x, y), camera, scene, &lights, rng, config);
    }

    img
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions concurrently.
pub fn parallel_render(
    camera: &Camera,
    scene: &Scene<impl Object>,
    config: &RenderConfig,
) -> image::RgbImage {
    let lights = if config.direct_lighting {
        scene.lights().collect::<Vec<_>>()
    } else {
        vec![]
    };

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
                &lights,
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
pub fn render_pixel<O: Object>(
    (x, y): (u32, u32),
    camera: &Camera,
    scene: &Scene<O>,
    lights: &[&O],
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Rgb<u8> {
    let mut c = (0..config.samples)
        .map(|_| {
            let r = camera.cast_ray((x, y), (config.width, config.height), rng);
            sample(&scene, lights, &r, 0, rng, config)
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

fn sample<O: Object, R: Rng>(
    scene: &Scene<O>,
    lights: &[&O],
    ray: &Ray,
    depth: u32,
    rng: &mut R,
    config: &RenderConfig,
) -> Vec3 {
    let sample_light = |light: &O, intersection: Vec3, n, rng: &mut R| {
        let (mut light_pos, light_radius) = light.bounding_sphere();

        if config.soft_shadows {
            light_pos = loop {
                let x = rng.gen::<f64>() * 2.0 - 1.0;
                let y = rng.gen::<f64>() * 2.0 - 1.0;
                if x.powi(2) + y.powi(2) <= 1.0 {
                    let l = (light_pos - ray.origin).normalized();
                    let u = l.cross(&Vec3::random_unit(rng)).normalized();
                    let v = l.cross(&u);

                    break light_pos + (u * (x * light_radius)) + (v * (y * light_radius));
                }
            };
        }

        let light_ray = Ray::new(intersection, (light_pos - intersection).normalized());

        // if `light_ray` goes in the opposite direction wrt `n` then it doesn't
        // reach the light for sure
        let diffuse = light_ray.dir.dot(&n);
        if diffuse <= 0.0 {
            return Vec3::zero();
        }

        // check if `intersection` is in the shadow of another object or reaches
        // a light
        if let Some((o, _t)) = scene.intersection(&light_ray) {
            if let Material::Light { emittance } = o.material() {
                return *emittance * diffuse;
            }
        }

        Vec3::zero()
    };

    let mut sample_material = |material: &Material, intersection, n| match *material {
        Material::Lambertian { albedo } => {
            let indirect = sample(
                scene,
                lights,
                &lambertian_bounce(intersection, n, rng),
                depth + 1,
                rng,
                config,
            );

            let direct = lights
                .iter()
                .map(|l| sample_light(l, intersection, n, rng))
                .sum::<Vec3>();

            albedo * (direct + indirect)
        }
        Material::Metal { albedo, fuzziness } => {
            let r = metal_bounce(ray, intersection, n, fuzziness, rng);

            if r.dir.dot(&n) < 0.0 {
                return Vec3::zero();
            }

            albedo * sample(scene, lights, &r, depth + 1, rng, config)
        }
        Material::Dielectric { refraction_index } => sample(
            scene,
            lights,
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

            sample_material(&s.material(), intersection, n)
        }

        None => sample_environment(scene, ray),
    }
}

fn sample_environment(scene: &Scene<impl Object>, ray: &Ray) -> Vec3 {
    match scene.environment {
        Environment::Color(c) => c,
        Environment::LinearGradient(a, b) => {
            let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
            vec3::lerp(a, b, t)
        }
    }
}

impl<T> Object for Box<T>
where
    T: Object + ?Sized,
{
    fn material(&self) -> &Material {
        self.deref().material()
    }

    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.deref().normal_at(p)
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.deref().bounding_sphere()
    }
}
