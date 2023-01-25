use geo::{
    primitive::polyline::Polyline, ray::Ray, sdf::Sdf, spatial_index::Shape, v3, Aabb, Axis, Vec3,
};

use crate::Object;

use rayon::prelude::*;

/// Shade the given `Sdf` by drawing the paths obtained by slicing the shape a
/// given amount of times along a given `Axis`.
///
/// In order to give a sense of shape and shadows a simple trick is used: slice
/// the shape
///
/// returning the paths only for the areas that get increasingly darker.
///
#[derive(Debug)]
pub struct SdfSlicer {
    sdf: Sdf,
    divs: u16,
    light_dir: Vec3,
    axis: Axis,
}

impl SdfSlicer {
    /// Create a `SdfSlicer` to slice the given `Sdf` `divs` times along the
    /// `axis`.
    ///
    /// The `light_dir` parameter is the direction of the light and it's used to
    /// determine how black a pixel is.
    pub fn new(sdf: Sdf, divs: u16, axis: Axis, light_dir: Vec3) -> Self {
        Self {
            sdf,
            divs,
            light_dir: light_dir.normalized(),
            axis,
        }
    }
}

impl Shape for SdfSlicer {
    type Intersection = f64;

    fn intersection(&self, ray: &geo::ray::Ray) -> Option<Self::Intersection> {
        self.sdf.ray_march(ray, 128)
    }

    fn bbox(&self) -> Aabb {
        self.sdf.bbox()
    }
}

impl Object for SdfSlicer {
    fn paths(&self) -> Vec<Polyline> {
        let bbox = self.bbox();

        (0..self.divs)
            .into_par_iter()
            .flat_map(|t| {
                let mut paths = vec![];

                let y = bbox.min()[self.axis]
                    + bbox.dimensions()[self.axis] * f64::from(t) / f64::from(self.divs - 1);
                let f = SdfField::new(&self.sdf, y, self.axis);

                let contours = marching_squares::march(&f, 0.0);
                for c in contours {
                    let points = c
                        .into_iter()
                        .map(|(x, y)| {
                            let p = f.to_3d(x, y);

                            let d = (bbox.center() - p).normalized();
                            let t = self.sdf.ray_march(&Ray::new(p, d), 10).unwrap_or(0.0);
                            p + d * t
                        })
                        .collect::<Polyline>();

                    let mut cur = Polyline::new();
                    for p in points.chop(0.01).iter() {
                        let n = self.sdf.normal_at(p);
                        let lt = n.dot(-self.light_dir);

                        // TODO: make this threshold configurable?
                        // TODO: shadows?
                        let black = lt <= 0.5 + 0.5 * ((t % 4) as f64 / 3.0);
                        if black {
                            cur.push(p);
                        } else if !cur.is_empty() {
                            paths.push(cur);
                            cur = Polyline::new();
                        }
                    }

                    if !cur.is_empty() {
                        paths.push(cur);
                    }
                }

                paths
            })
            .collect()
    }
}

pub struct SdfField {
    data: Vec<f64>,
    y: f64,
    axis: Axis,
    width: usize,
    height: usize,
    bbox: Aabb,
}

impl SdfField {
    /// How many cells per unit to use when calculating the paths.
    const RESOLUTION: f64 = 10.0;

    pub fn new(sdf: &Sdf, y: f64, axis: Axis) -> Self {
        const EPS: f64 = 0.001;

        let bbox = sdf.bbox();
        let dims = bbox.dimensions();

        let (w, h) = match axis {
            Axis::X => (dims.y, dims.z),
            Axis::Y => (dims.z, dims.x),
            Axis::Z => (dims.x, dims.y),
        };
        let w = (w.ceil() * Self::RESOLUTION) as usize;
        let h = (h.ceil() * Self::RESOLUTION) as usize;

        // +1 to include the last pixel, +2 to add a black border
        let (w, h) = (w + 1 + 2, h + 1 + 2);

        let mut res = Self {
            bbox: sdf.bbox(),
            data: vec![1.0; w * h],
            y,
            axis,
            width: w,
            height: h,
        };

        // For marching squares we're not actually interested in having all the
        // correct values being returned, but we just need to distinguish
        // between what's inside and what's outside.
        //
        // This means we can use ray-marching to quickly jump when we're outside
        // of the Sdf. However, when we reach the interior we have to check
        // pixel by pixel given that usually the interior of an SDF is broken.
        //
        // Note that we actually calculate all the distances on the "boundary"
        // of the Sdf (both inside and outside) to have nicely interpolated
        // paths. Otherwise, the paths are too jaggy and not that pretty.
        for j in 0..h {
            let mut i = 0.0;
            while i < w as f64 {
                let d = sdf.dist(&res.to_3d(i, j as f64));

                if d <= EPS {
                    res.data[j * w + i as usize] = d;

                    // we're inside, populate the pixels to the left and above
                    // if they're outside the Sdf (if they're inside we already
                    // calculated their value).
                    if i > 0.0 && res.data[j * w + i as usize - 1] > EPS {
                        res.data[j * w + i as usize - 1] = sdf.dist(&res.to_3d(i - 1.0, j as f64));
                    }

                    if j > 0 && res.data[(j - 1) * w + i as usize] > EPS {
                        res.data[(j - 1) * w + i as usize] =
                            sdf.dist(&res.to_3d(i, j as f64 - 1.0));
                    }

                    i += 1.0;
                    continue;
                }

                // we're outside, but the pixel to the left or above are inside
                // the Sdf, calculate the distance to have nicely interpolated
                // paths anyway
                if (i > 0.0 && res.data[j * w + i as usize - 1] <= EPS)
                    || j > 0 && res.data[(j - 1) * w + i as usize] <= EPS
                {
                    res.data[j * w + i as usize] = d;
                }

                i += (d * Self::RESOLUTION).floor().max(1.0);
            }
        }

        res
    }

    /// Un-project the given 2D point coming from the slice to the original 3D.
    pub fn to_3d(&self, i: f64, j: f64) -> Vec3 {
        let mut p = self.bbox.min();
        p -= 1.0 / Self::RESOLUTION;
        p[self.axis] = self.y;

        p += match self.axis {
            Axis::X => v3(0, i, j),
            Axis::Y => v3(j, 0, i),
            Axis::Z => v3(i, j, 0),
        } / v3(Self::RESOLUTION, Self::RESOLUTION, Self::RESOLUTION);

        p
    }
}

impl marching_squares::Field for SdfField {
    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn z_at(&self, x: usize, y: usize) -> f64 {
        self.data[y * self.width + x]
    }
}
