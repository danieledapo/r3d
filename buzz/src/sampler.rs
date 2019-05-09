use rand::prelude::*;

use geo::ray::Ray;
use geo::spatial_index::Intersection;
use geo::{vec3, Vec3};

use crate::material::{dielectric_bounce, lambertian_bounce, metal_bounce, Material};
use crate::{Environment, Object, RenderConfig, Scene};

pub fn sample<'s, O: Object<'s> + 's>(
    scene: &'s Scene<O>,
    lights: &[&O],
    ray: &Ray,
    depth: u32,
    rng: &mut impl Rng,
    config: &RenderConfig,
) -> Vec3 {
    match scene.intersection(ray) {
        Some(_) if depth >= config.max_bounces => Vec3::zero(),
        Some((s, hit)) => {
            let intersection = ray.point_at(hit.t());
            let n = s.normal_at(intersection);

            sample_material(
                scene,
                lights,
                &ray,
                depth,
                &s.material(),
                intersection,
                n,
                rng,
                config,
            )
        }

        None => sample_environment(scene, &ray),
    }
}

fn sample_material<'s, O: Object<'s> + 's>(
    scene: &'s Scene<O>,
    lights: &[&O],
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
                .map(|l| sample_light(scene, l, ray, intersection, n, config, rng))
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

fn sample_light<'s, O: Object<'s> + 's>(
    scene: &'s Scene<O>,
    light: &O,
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

fn sample_environment<'s, O: Object<'s> + 's>(scene: &Scene<O>, ray: &Ray) -> Vec3 {
    match scene.environment {
        Environment::Color(c) => c,
        Environment::LinearGradient(a, b) => {
            let t = 0.5 * (ray.dir.y / ray.dir.norm() + 1.0);
            vec3::lerp(a, b, t)
        }
    }
}
