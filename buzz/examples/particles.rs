//! See <https://github.com/d-dorazio/dla> for an example on how to generate the particles.

use std::io::{self, BufRead, BufReader};

use geo::{util::opener, Aabb, Vec3};

use buzz::*;

pub fn main() -> opener::Result<()> {
    let input = BufReader::new(io::stdin());
    let spheres = input
        .lines()
        .map(|l| {
            let l = l.unwrap();

            let mut coords = l.split(',');
            let x = coords.next().unwrap().parse::<f64>().unwrap();
            let y = coords.next().unwrap().parse::<f64>().unwrap();
            let z = coords.next().unwrap().parse::<f64>().unwrap();

            SphereGeometry::new(Vec3::new(x, y, z), 1.73)
        })
        .collect::<Vec<_>>();

    // scale input in [-1,1] range so that camera positioning is easy
    let bbox = Aabb::from_iter(spheres.iter().map(|s| s.center))
        .unwrap_or_else(|| Aabb::new(Vec3::zero()));

    let mut objects = SceneObjects::new();
    for mut s in spheres {
        let Vec3 { x: w, y: h, z: d } = bbox.dimensions();

        s.center = (s.center - bbox.min()) / bbox.dimensions() * 2.0 - 1.0;
        s.radius /= w.min(h).min(d);

        let c = Vec3::lerp(
            Vec3::new(0.34, 0.7, 0.03),
            Vec3::new(0.85, 0.84, 0.0),
            s.center.norm2(),
        );

        objects.push(SimpleObject::new(s, Material::lambertian(c)))
    }

    // lights
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(3.1, 0.0, 2.8), 0.6),
        Material::light(Vec3::replicate(0.4)),
    ));
    objects.push(SimpleObject::new(
        SphereGeometry::new(Vec3::new(-3.1, 0.0, 2.8), 0.2),
        Material::light(Vec3::replicate(0.1)),
    ));

    let scene = Scene::new(objects, Environment::Color(Vec3::zero()));

    let camera = Camera::look_at(
        Vec3::new(0.0, 0.0, 2.0),
        Vec3::zero(),
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
    );

    let img = parallel_render(
        &camera,
        &scene,
        &RenderConfig {
            width: 4096,
            height: 4096,
            samples: 20,
            max_bounces: 10,
            direct_lighting: true,
            soft_shadows: false,
        },
    );
    img.save("particles.png").expect("cannot save output image");

    opener::open("particles.png")
}
