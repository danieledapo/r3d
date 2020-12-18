#![allow(clippy::useless_let_if_seq)]

pub mod camera;
pub mod csg;
pub mod cube;
pub mod cylinder;
pub mod facet;
pub mod material;
pub mod plane;
pub mod sphere;
pub mod transformed;

mod sampler;

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use image::{Rgb, RgbImage};
use rand::prelude::*;
use rayon::prelude::*;

use geo::ray::Ray;
use geo::spatial_index::Bvh;
use geo::spatial_index::{Intersection, Shape};
use geo::{Aabb, Vec3};

use camera::Camera;
use material::Material;

/// An `Hit` represents an intersection between a `Ray` and the shapes in a `Scene`.
#[derive(Debug)]
pub struct Hit {
    /// `t` parameter wrt the `Ray` that generated this `Hit`
    pub t: f64,

    /// the `Surface` the `Ray` hit
    pub surface_id: usize,

    pub point_and_normal: Option<(Vec3, Vec3)>,
}

impl Hit {
    pub fn new(t: f64, point_and_normal: Option<(Vec3, Vec3)>) -> Self {
        Self {
            t,
            point_and_normal,
            surface_id: 0,
        }
    }
}

/// An `Object` that can be rendered.
pub trait Object: Shape<Intersection = Hit> + Surface + Sync + Send {
    /// Getter for the `Material` the `Object` is made of.
    fn material(&self) -> &Material;

    fn set_surface_id(&mut self, id: usize);
}

pub trait Surface: std::fmt::Debug {
    /// Calculate the normal for the given point `p`. This method should never
    /// be called if the `Surface` does not intersect it.
    fn normal_at(&self, p: Vec3) -> Vec3;
}

#[derive(Debug)]
pub struct SceneObjects {
    objects: Vec<Arc<dyn Object>>,
}

/// A `Scene` is a collection of objects that can be rendered.
#[derive(Debug)]
pub struct Scene {
    objects: Vec<Arc<dyn Object>>,
    objects_index: Bvh<Arc<dyn Object>>,
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
    pub fn new(objects: SceneObjects, environment: Environment) -> Self {
        let objects: Vec<Arc<_>> = objects.objects;
        let objects_index: Bvh<_> = objects.iter().cloned().collect();
        Scene {
            objects,
            objects_index,
            environment,
        }
    }

    /// Calculate the intersection between a `Ray` and all the objects in the
    /// scene returning the closest object (along with its intersection result)
    /// to the ray.
    pub fn intersection(&self, ray: &Ray) -> Option<(&dyn Object, Hit)> {
        self.objects_index
            .intersections(ray)
            .min_by(|(_, t0), (_, t1)| t0.t().partial_cmp(&t1.t()).unwrap())
            .map(|(s, t)| (s.as_ref(), t))
    }

    pub fn surface(&self, sfid: usize) -> &dyn Object {
        self.objects[sfid].as_ref()
    }

    /// Return an iterator over all the lights in the `Scene`.
    pub fn lights(&self) -> impl Iterator<Item = &dyn Object> {
        self.objects
            .iter()
            .filter(|o| {
                if let Material::Light { .. } = o.material() {
                    true
                } else {
                    false
                }
            })
            .map(|o| o.as_ref())
    }
}

impl SceneObjects {
    pub fn new() -> Self {
        Self { objects: vec![] }
    }

    pub fn push(&mut self, mut o: impl Object + 'static) {
        o.set_surface_id(self.objects.len());
        self.objects.push(Arc::new(o));
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
pub fn render(camera: &Camera, scene: &Scene, config: &RenderConfig) -> image::RgbImage {
    let lights = if config.direct_lighting {
        scene.lights().collect::<Vec<_>>()
    } else {
        vec![]
    };

    let mut rng = thread_rng();
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

    img.par_chunks_mut(3)
        .zip((0_u32..config.width * config.height).into_par_iter())
        .for_each(|(pix, i)| {
            let x = i % config.width;
            let y = i / config.width;

            let Rgb([r, g, b]) = render_pixel(
                (x as u32, y as u32),
                camera,
                scene,
                &lights,
                &mut rand::thread_rng(),
                config,
            );

            pix[0] = r;
            pix[1] = g;
            pix[2] = b;
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
            sampler::sample(scene, lights, &r, 0, rng, config)
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

impl Intersection for Hit {
    fn t(&self) -> f64 {
        self.t
    }
}

impl<T> Object for Box<T>
where
    T: Object + ?Sized,
{
    fn material(&self) -> &Material {
        self.deref().material()
    }

    fn set_surface_id(&mut self, id: usize) {
        self.deref_mut().set_surface_id(id)
    }
}
impl<T> Surface for Box<T>
where
    T: Surface + ?Sized,
{
    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.deref().normal_at(p)
    }
}

#[derive(Debug)]
pub struct SimpleObject<S> {
    geom: S,
    material: Material,
    surface_id: usize,
}

impl<G> SimpleObject<G> {
    pub fn new(geom: G, material: Material) -> Self {
        SimpleObject {
            geom,
            material,
            surface_id: 0,
        }
    }
}

impl<S> Object for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Surface + Sync + Send,
{
    fn material(&self) -> &Material {
        &self.material
    }

    fn set_surface_id(&mut self, id: usize) {
        self.surface_id = id;
    }
}

impl<S> Surface for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Surface + Sync + Send,
{
    fn normal_at(&self, p: Vec3) -> Vec3 {
        self.geom.normal_at(p)
    }
}

impl<S> Shape for SimpleObject<S>
where
    S: Shape<Intersection = Hit> + Sync + Send,
{
    type Intersection = Hit;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let mut h = self.geom.intersection(ray)?;
        h.surface_id = self.surface_id;
        Some(h)
    }

    fn bbox(&self) -> Aabb {
        self.geom.bbox()
    }

    fn bounding_sphere(&self) -> (Vec3, f64) {
        self.geom.bounding_sphere()
    }
}
