use std::sync::Arc;

use rand::prelude::*;

use geo::{primitive::polyline::Polyline, spatial_index::Shape, util::opener, Aabb, Vec3};

use l::*;

#[derive(Debug, Clone)]
pub struct Building {
    cube: Cube,
    steps: u8,
}

impl Building {
    pub fn new(aabb: Aabb, steps: u8) -> Self {
        Self {
            cube: Cube::new(aabb),
            steps,
        }
    }
}

impl Shape for Building {
    type Intersection = f64;

    fn intersection(&self, ray: &geo::ray::Ray) -> Option<Self::Intersection> {
        self.cube.intersection(ray)
    }

    fn bbox(&self) -> Aabb {
        self.cube.bbox()
    }
}

impl Object for Building {
    fn paths(&self) -> Vec<Polyline> {
        let mut out = self.cube.paths();

        let Vec3 {
            x: x0,
            y: y0,
            z: z0,
        } = self.bbox().min();
        let Vec3 {
            x: x1,
            y: y1,
            z: z1,
        } = self.bbox().max();

        for i in 0..self.steps {
            let t = f64::from(i + 1) / f64::from(self.steps + 1);
            let dx = (x1 - x0) * t;
            let dy = (y1 - y0) * t;
            let dz = (z1 - z0) * t;

            out.push(vec![Vec3::new(x0 + dx, y0, z0), Vec3::new(x0 + dx, y0, z1)].into());
            out.push(vec![Vec3::new(x0 + dx, y1, z0), Vec3::new(x0 + dx, y1, z1)].into());

            out.push(vec![Vec3::new(x0, y0 + dy, z0), Vec3::new(x0, y0 + dy, z1)].into());
            out.push(vec![Vec3::new(x1, y0 + dy, z0), Vec3::new(x1, y0 + dy, z1)].into());

            out.push(
                vec![
                    Vec3::new(x0, y0, z0 + dz),
                    Vec3::new(x1, y0, z0 + dz),
                    Vec3::new(x1, y1, z0 + dz),
                    Vec3::new(x0, y1, z0 + dz),
                    Vec3::new(x0, y0, z0 + dz),
                ]
                .into(),
            );
        }

        out
    }
}

pub fn main() -> opener::Result<()> {
    let mut objects = vec![];

    let mut rng = thread_rng();
    for y in -10..=10 {
        for x in -10..=10 {
            if rng.gen::<f64>() >= 0.8 {
                continue;
            }

            let height = rng.gen_range(1..10_u8);

            let building = Building::new(
                Aabb::with_dimensions(
                    Vec3::new(f64::from(x) - 0.5, f64::from(y) - 0.5, 0.0),
                    Vec3::new(0.8, 0.8, height.into()),
                ),
                height,
            );

            objects.push(Arc::new(building) as Arc<dyn Object>);
        }
    }

    let scene = Scene::new(objects);

    let camera = Camera::look_at(
        Vec3::new(5.0, -3.0, 20.0),
        Vec3::zero(),
        Vec3::new(0.0, 0.0, -1.0),
    )
    .with_perspective_projection(45.0, 1.0, 0.01, 100.0);

    let paths = render(
        camera,
        &scene,
        &Settings {
            chop_eps: 0.001,
            simplify_eps: 0.001,
        },
    );
    dump_svg("skyscrapers.svg", &paths, SvgSettings::new(2048.0, 2048.0))
        .expect("cannot save skyscrapers.svg");

    opener::open("skyscrapers.svg")
}
