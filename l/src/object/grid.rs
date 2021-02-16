use geo::{
    ray::Ray,
    spatial_index::{Bvh, Intersection, Shape},
    Aabb, Triangle, Vec3,
};

use crate::{Facet, Object, Polyline};

#[derive(Debug)]
pub struct Grid {
    bvh: Bvh<Facet>,
    heightmap: Vec<Vec3>,
    steps: usize,
    bbox: Aabb,
}

impl Grid {
    pub fn from_fn(
        (sx, sy): (f64, f64),
        (ex, ey): (f64, f64),
        steps: u16,
        fun: impl Fn(f64, f64) -> f64,
    ) -> Self {
        assert!(steps >= 2);

        let steps = usize::from(steps);
        let mut heightmap = Vec::with_capacity(steps.pow(2));

        for i in 0..steps {
            let y = sy + (ey - sy) * (i as f64) / (steps as f64 - 1.0);
            for j in 0..steps {
                let x = sx + (ex - sx) * (j as f64) / (steps as f64 - 1.0);

                heightmap.push(Vec3::new(x, y, fun(x, y)));
            }
        }

        let mut bbox = Aabb::new(heightmap[0]);
        let mut facets = Vec::with_capacity(steps.pow(2));

        for y in 0..steps - 1 {
            for x in 0..steps - 1 {
                let a = heightmap[y * steps + x];
                let b = heightmap[y * steps + x + 1];
                let c = heightmap[(y + 1) * steps + x];
                let d = heightmap[(y + 1) * steps + x + 1];

                bbox.extend(&[a, b, c, d]);
                facets.push(Facet::new(Triangle::new(a, c, b)));
                facets.push(Facet::new(Triangle::new(b, c, d)));
            }
        }

        Self {
            heightmap,
            steps,
            bbox,
            bvh: facets.into_iter().collect(),
        }
    }
}

impl Shape for Grid {
    type Intersection = f64;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        self.bvh
            .intersections(ray)
            .map(|(_, t)| t)
            .min_by(|t0, t1| t0.t().partial_cmp(&t1.t()).unwrap())
    }

    fn bbox(&self) -> Aabb {
        self.bbox.clone()
    }
}

impl Object for Grid {
    fn paths(&self) -> Vec<Polyline> {
        let mut out = Vec::with_capacity(2 * self.steps);

        for i in 0..self.steps {
            out.push(self.heightmap[i * self.steps..(i + 1) * self.steps].to_vec());
            out.push(
                (0..self.steps)
                    .map(|j| self.heightmap[j * self.steps + i])
                    .collect(),
            );
        }

        out
    }
}
