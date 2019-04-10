use geo::vec3::Vec3;

/// Enum over all the supported `Material`s. Each variant dictates how light
/// interacts(reflects, refracts, etc..) with them. They're mainly composed of
/// an `albedo` field which is the intrinsic color of the material.
#[derive(Debug, PartialEq, Clone)]
pub enum Material {
    /// The `Lambertian` material is a perfectly matte or diffuse surface which
    /// is modeled after the [Lambertian reflectance model][0].
    ///
    /// [0]: https://en.wikipedia.org/wiki/Lambertian_reflectance
    Lambertian { albedo: Vec3 },

    /// A metallic material that perfectly reflects light as it comes in.
    Metal { albedo: Vec3 },
}
