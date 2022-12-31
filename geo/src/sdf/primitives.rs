use crate::{sphere, Aabb, Vec3};

use super::Sdf;

pub fn sphere(radius: f64) -> Sdf {
    Sdf::from_fn(sphere::bounding_box(Vec3::zero(), radius), move |p| {
        p.norm() - radius
    })
}

pub fn cuboid(size: Vec3) -> Sdf {
    let d = size / 2.0;
    Sdf::from_fn(Aabb::new(-d).expanded(d), move |p| {
        let mut x = if p.x < 0.0 { -p.x } else { p.x };
        let mut y = if p.y < 0.0 { -p.y } else { p.y };
        let mut z = if p.z < 0.0 { -p.z } else { p.z };

        x -= size.x / 2.0;
        y -= size.y / 2.0;
        z -= size.z / 2.0;

        let a = x.max(y).max(z).min(0.0);

        x = x.max(0.0);
        y = y.max(0.0);
        z = z.max(0.0);

        let b = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
        a + b
    })
}

pub fn cylinder(radius: f64, height: f64) -> Sdf {
    let bbox = Aabb::new(Vec3::new(-radius, -height / 2.0, -radius)).expanded(Vec3::new(
        radius,
        height / 2.0,
        radius,
    ));

    Sdf::from_fn(bbox, move |p| {
        let mut x = (p.x.powi(2) + p.z.powi(2)).sqrt();
        if x < 0.0 {
            x = -x;
        }

        let mut y = if p.y < 0.0 { -p.y } else { p.y };

        x -= radius;
        y -= height / 2.0;
        let a = x.max(y).min(0.0);

        x = x.max(0.0);
        y = y.max(0.0);
        let b = (x.powi(2) + y.powi(2)).sqrt();

        a + b
    })
}

pub fn torus(r1: f64, r2: f64) -> Sdf {
    let a = r1;
    let b = r1 + r2;

    let bbox = Aabb::new(Vec3::new(-b, -b, -a)).expanded(Vec3::new(b, b, a));

    Sdf::from_fn(bbox, move |p| {
        let q = Vec3::new(Vec3::new(p.x, p.y, 0.0).norm() - r2, p.z, 0.0);
        q.norm() - r1
    })
}

pub fn capsule(a: Vec3, b: Vec3, r: f64) -> Sdf {
    let bbox = Aabb::new(a - r)
        .expanded(a + r)
        .expanded(b - r)
        .expanded(b + r);

    Sdf::from_fn(bbox, move |p| {
        let pa = *p - a;
        let ba = b - a;
        let h = f64::clamp(pa.dot(ba) / ba.dot(ba), 0.0, 1.0);
        (pa - ba * h).norm() - r
    })
}

pub fn octahedron(r: f64) -> Sdf {
    let tan_30 = f64::to_radians(30.0).tan();
    Sdf::from_fn(
        Aabb::new(Vec3::replicate(-r)).expanded(Vec3::replicate(r)),
        move |p| {
            let m = p.x.abs() + p.y.abs() + p.z.abs() - r;
            m * tan_30
        },
    )
}
