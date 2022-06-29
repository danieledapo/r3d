use geo::{
    primitive::polyline::Polyline,
    ray::Ray,
    sdf::{self, Sdf},
    spatial_index::Shape,
    util::arange,
    Aabb, Vec3,
};
use marching_squares::Field;

use crate::Object;

#[derive(Debug)]
pub struct ZSlicer<S> {
    sdf: S,
    zstep: f64,
    quantization: f64,
}

impl<S: Sdf> ZSlicer<S> {
    pub fn sdf(step: f64, sdf: S) -> Self {
        Self {
            zstep: step,
            sdf,
            quantization: 0.1,
        }
    }

    pub fn with_zstep(mut self, step: f64) -> Self {
        self.zstep = step;
        self
    }

    pub fn with_quantization_step(mut self, quantization: f64) -> Self {
        self.quantization = quantization;
        self
    }
}

impl<S: Sdf> Shape for ZSlicer<S> {
    type Intersection = f64;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        sdf::ray_marching(&self.sdf, ray, 100)
    }

    fn bbox(&self) -> geo::Aabb {
        self.sdf.bbox()
    }
}

impl<S: Sdf + Sync + Send> Object for ZSlicer<S> {
    fn paths(&self) -> Vec<Polyline> {
        struct Level<'a, S> {
            bbox: Aabb,
            sdf: &'a S,
            z: f64,
            quantization: f64,
        }

        impl<S: Sdf> Field for Level<'_, S> {
            fn dimensions(&self) -> (usize, usize) {
                let dims = self.bbox.dimensions();
                (
                    ((dims.x + 2.0) / self.quantization).ceil() as usize,
                    ((dims.y + 2.0) / self.quantization).ceil() as usize,
                )
            }

            fn z_at(&self, x: usize, y: usize) -> f64 {
                let x = self.bbox.min().x - 1.0 + (x as f64 * self.quantization);
                let y = self.bbox.min().y - 1.0 + (y as f64 * self.quantization);

                -self.sdf.dist(&Vec3::new(x, y, self.z))
            }
        }

        let bbox = self.sdf.bbox();
        let mut paths = vec![];

        for z in arange(bbox.min().z, bbox.max().z, self.zstep) {
            let level = Level {
                bbox: bbox.clone(),
                sdf: &self.sdf,
                z,
                quantization: self.quantization,
            };

            let contours = marching_squares::march(&level, 0.0);

            paths.extend(contours.into_iter().map(|path| {
                path.into_iter()
                    .map(|(x, y)| {
                        Vec3::new(
                            bbox.min().x - 1.0 + x * self.quantization,
                            bbox.min().y - 1.0 + y * self.quantization,
                            z,
                        )
                    })
                    .collect::<Polyline>()
            }));
        }

        paths
    }
}
