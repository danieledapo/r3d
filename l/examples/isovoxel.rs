use std::{collections::HashMap, sync::Arc};

use rand::prelude::*;

use geo::{primitive::polyline::Polyline, spatial_index::Shape, util::opener, Aabb, Vec3};
use l::*;

#[derive(Debug)]
pub struct IsoVoxel {
    cube: Cube,
}

impl IsoVoxel {
    pub fn new(bbox: Aabb) -> Self {
        IsoVoxel {
            cube: Cube::new(bbox),
        }
    }
}

impl Shape for IsoVoxel {
    type Intersection = <Cube as Shape>::Intersection;

    fn intersection(&self, ray: &geo::ray::Ray) -> Option<Self::Intersection> {
        self.cube.intersection(ray)
    }

    fn bbox(&self) -> Aabb {
        self.cube.bbox()
    }
}

impl Object for IsoVoxel {
    fn paths(&self) -> Vec<Polyline> {
        let bbox = self.bbox();
        let Vec3 {
            x: x0,
            y: y0,
            z: z0,
        } = bbox.min();
        let Vec3 {
            x: x1,
            y: y1,
            z: z1,
        } = bbox.max();

        vec![
            // top
            vec![
                Vec3::new(x0, y1, z0),
                Vec3::new(x1, y1, z0),
                Vec3::new(x1, y1, z1),
                Vec3::new(x0, y1, z1),
                Vec3::new(x0, y1, z0),
            ]
            .into(),
            // front
            vec![
                Vec3::new(x0, y0, z1),
                Vec3::new(x1, y0, z1),
                Vec3::new(x1, y1, z1),
                Vec3::new(x0, y1, z1),
                Vec3::new(x0, y0, z1),
            ]
            .into(),
            // right
            vec![
                Vec3::new(x1, y0, z0),
                Vec3::new(x1, y0, z1),
                Vec3::new(x1, y1, z1),
                Vec3::new(x1, y1, z0),
                Vec3::new(x1, y0, z0),
            ]
            .into(),
        ]
    }
}

pub fn main() -> opener::Result<()> {
    // first dedup cubes by their projected position so that it's easier on the
    // spatial index
    let mut cubes = HashMap::new();

    let mut rng = thread_rng();
    for z in -10..=10 {
        for y in -10..=10 {
            for x in -10..=10 {
                if rng.gen::<f64>() <= 0.98 {
                    continue;
                }

                cubes.insert((x - z, y - z), (x, y, z));
            }
        }
    }

    let objects: Vec<_> = cubes
        .values()
        .map(|&(a, b, c)| {
            Arc::new(Cube::new(Aabb::cube(
                Vec3::new(a.into(), b.into(), c.into()),
                1.0,
            ))) as Arc<dyn Object>
        })
        .collect();

    let scene = Scene::new(objects);

    let camera = Camera::look_at(
        Vec3::new(50.0, 50.0, 50.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
    )
    .with_isometric_projection(20.0, 1.0, 0.01, 1000.0);
    // .with_perspective_projection(90.0, 1.0, 0.01, 1000.0);

    let paths = render(
        &camera,
        &scene,
        &Settings {
            chop_eps: 0.0005,
            simplify_eps: 0.001,
        },
    );
    dump_svg("isovoxels.svg", &paths, SvgSettings::new(2048.0, 2048.0))
        .expect("cannot save isovoxels.svg");

    // opener::open("isovoxels.svg")
    Ok(())
}
