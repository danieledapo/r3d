use geo::{primitive::polyline::Polyline, ray::Ray, spatial_index::Shape, v3, Aabb, Vec3};

use crate::Object;

#[derive(Debug, Clone)]
pub struct Cube {
    bbox: Aabb,
}

impl Cube {
    pub fn new(bbox: Aabb) -> Self {
        Cube { bbox }
    }
}

impl Shape for Cube {
    type Intersection = f64;

    fn intersection(&self, ray: &Ray) -> Option<Self::Intersection> {
        let (t0, t1) = self.bbox.ray_intersection(ray)?;
        if t0 >= t1 || t1 < 0.0 {
            None
        } else {
            Some(t0.max(0.0))
        }
    }

    fn bbox(&self) -> Aabb {
        self.bbox.clone()
    }
}

impl Object for Cube {
    fn paths(&self) -> Vec<Polyline> {
        let Vec3 {
            x: x0,
            y: y0,
            z: z0,
        } = self.bbox.min();
        let Vec3 {
            x: x1,
            y: y1,
            z: z1,
        } = self.bbox.max();

        vec![
            // left
            vec![
                v3(x0, y0, z0),
                v3(x0, y0, z1),
                v3(x0, y1, z1),
                v3(x0, y1, z0),
                v3(x0, y0, z0),
            ]
            .into(),
            // right
            vec![
                v3(x1, y0, z0),
                v3(x1, y0, z1),
                v3(x1, y1, z1),
                v3(x1, y1, z0),
                v3(x1, y0, z0),
            ]
            .into(),
            // back
            vec![
                v3(x0, y0, z0),
                v3(x1, y0, z0),
                v3(x1, y1, z0),
                v3(x0, y1, z0),
                v3(x0, y0, z0),
            ]
            .into(),
            // front
            vec![
                v3(x0, y0, z1),
                v3(x1, y0, z1),
                v3(x1, y1, z1),
                v3(x0, y1, z1),
                v3(x0, y0, z1),
            ]
            .into(),
            // bottom
            vec![
                v3(x0, y0, z0),
                v3(x1, y0, z0),
                v3(x1, y0, z1),
                v3(x0, y0, z1),
                v3(x0, y0, z0),
            ]
            .into(),
            // top
            vec![
                v3(x0, y1, z0),
                v3(x1, y1, z0),
                v3(x1, y1, z1),
                v3(x0, y1, z1),
                v3(x0, y1, z0),
            ]
            .into(),
        ]
    }
}
