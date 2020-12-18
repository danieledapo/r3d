use geo::{ray::Ray, spatial_index::Intersection, Vec3};

use std::convert::TryFrom;

use image::{Rgb, RgbImage};
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use rayon::prelude::*;

use crate::{
    material::{dielectric_bounce, lambertian_bounce, metal_bounce, Material},
    Camera, Environment, Object, Scene,
};

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
pub fn render(camera: &Camera, scene: &Scene, config: &RenderConfig) -> image::RgbImage {
    let lights = if config.direct_lighting {
        scene.lights().collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut rng = XorShiftRng::seed_from_u64(thread_rng().gen());
    let mut img = RgbImage::new(config.width, config.height);

    for (x, y, pix) in img.enumerate_pixels_mut() {
        *pix = render_pixel((x, y), camera, scene, &lights, &mut rng, config);
    }

    img
}

/// Render a `Scene` from a `Camera` to a new `RgbImage` of the given
/// dimensions concurrently.
pub fn parallel_render(camera: &Camera, scene: &Scene, config: &RenderConfig) -> image::RgbImage {
    let lights = if config.direct_lighting {
        scene.lights().collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut img = RgbImage::new(config.width, config.height);

    img.par_chunks_mut(3 * usize::try_from(config.width).unwrap())
        .zip((0_u32..config.height).into_par_iter())
        .for_each(|(row, y)| {
            let mut rng = XorShiftRng::seed_from_u64(thread_rng().gen());

            for (pix, x) in row.chunks_mut(3).zip(0..) {
                let Rgb([r, g, b]) = render_pixel((x, y), camera, scene, &lights, &mut rng, config);

                pix[0] = r;
                pix[1] = g;
                pix[2] = b;
            }
        });

    img
}

/// Render a single pixel of an image from a `Scene` and `Camera`.
pub fn render_pixel(
    (x, y): (u32, u32),
    camera: &Camera,
    scene: &Scene,
    lights: &[&dyn Object],
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Rgb<u8> {
    let mut c = (0..config.samples)
        .map(|_| {
            let r = camera.cast_ray((x, y), (config.width, config.height), rng);
            sample(scene, lights, &r, 0, rng, config)
        })
        .sum::<Vec3>()
        / f64::from(config.samples);

    // gamma correct pixels
    c.x = c.x.sqrt();
    c.y = c.y.sqrt();
    c.z = c.z.sqrt();

    Rgb([
        (c.x * 255.0) as u8,
        (c.y * 255.0) as u8,
        (c.z * 255.0) as u8,
    ])
}

pub fn sample(
    scene: &Scene,
    lights: &[&dyn Object],
    ray: &Ray,
    depth: u32,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Vec3 {
    match scene.intersection(ray) {
        Some(_) if depth >= config.max_bounces => Vec3::zero(),
        Some((s, hit)) => {
            let (intersection, n) = hit.point_and_normal.unwrap_or_else(|| {
                let intersection = ray.point_at(hit.t());
                let n = scene.surface(hit.surface_id).normal_at(intersection);

                (intersection, n)
            });

            sample_material(
                scene,
                lights,
                &ray,
                depth,
                s.material(),
                intersection,
                n,
                rng,
                config,
            )
        }

        None => sample_environment(scene, &ray),
    }
}

fn sample_material(
    scene: &Scene,
    lights: &[&dyn Object],
    ray: &Ray,
    depth: u32,
    material: &Material,
    intersection: Vec3,
    n: Vec3,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Vec3 {
    match *material {
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
                .map(|l| sample_light(scene, *l, ray, intersection, n, config, rng))
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
    }
}

fn sample_light(
    scene: &Scene,
    light: &dyn Object,
    ray: &Ray,
    intersection: Vec3,
    n: Vec3,
    config: &RenderConfig,
    rng: &mut impl Rng,
) -> Vec3 {
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
}

fn sample_environment(scene: &Scene, ray: &Ray) -> Vec3 {
    match scene.environment {
        Environment::Color(c) => c,
        Environment::LinearGradient(a, b) => {
            let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
            Vec3::lerp(a, b, t)
        }
    }
}
