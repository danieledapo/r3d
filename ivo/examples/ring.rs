use geo::{sdf::Sdf, util::opener, Aabb, Vec3};
use ivo::*;

#[derive(Debug)]
struct Ring {
    r1: f64,
    r2: f64,
}

impl Sdf for Ring {
    fn dist(&self, p: &Vec3) -> f64 {
        let q = Vec3::new(Vec3::new(p.x, p.y, 0.0).norm() - self.r2, p.z, 0.0);
        q.norm2() - self.r1
    }

    fn bbox(&self) -> Aabb {
        let a = self.r1;
        let b = self.r1 + self.r2;

        Aabb::new(Vec3::new(-b, -b, -a)).expanded(Vec3::new(b, b, a))
    }
}

pub fn main() {
    let mut scene = Scene::new();

    scene.sdf(&Ring {
        r1: 200.0,
        r2: 120.0,
    });

    let triangles = render(&scene);

    dump_svg("ring.svg", &triangles, &SvgSettings::new(1920.0, 1080.0))
        .expect("cannot save ring.svg");

    opener::open("ring.svg").expect("cannot open ring.svg");
}
