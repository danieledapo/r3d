use rand::Rng;

use geo::vec3::Vec3;

pub fn random_in_unit_circle(rng: &mut impl Rng) -> Vec3 {
    loop {
        let x = rng.gen();
        let y = rng.gen();
        let z = rng.gen();

        let v = Vec3::new(x, y, z) * 2.0 - 1.0;

        if v.norm2() < 1.0 {
            break v;
        }
    }
}
