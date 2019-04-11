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

    /// A metallic material that reflects light as it comes in. The `fuzziness`
    /// attribute is how much to perturbate each reflected ray. A low value of
    /// `fuzziness` makes it reflect more accurately because the reflected rays
    /// will change less. On the other hand, an high value will make it a bit
    /// opaque while still reflecting its surroundings.
    Metal { albedo: Vec3, fuzziness: f64 },

    /// Clear materials like glass and diamond are of type Dielectric and are
    /// identified by a refracion index. For example, glass has a refraction
    /// index in [1.3, 1.7] while diamond is 2.4.
    Dielectric { refraction_index: f64 },
}
