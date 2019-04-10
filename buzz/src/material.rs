use geo::vec3::Vec3;

#[derive(Debug, PartialEq, Clone)]
pub enum Material {
    Lambertian { albedo: Vec3 },
    Metal { albedo: Vec3 },
}
