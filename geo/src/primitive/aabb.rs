use crate::{
    mat4::{Mat4, Transform},
    ray::Ray,
    Vec3,
};

/// An [Axis aligned bounding box][0] useful for approximating the boundary of
/// shapes.
///
/// [0]:
/// https://en.wikipedia.org/wiki/Minimum_bounding_box#Axis-aligned_minimum_bounding_box
#[derive(Debug, Clone, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    /// Create a bounding box that covers a single point.
    pub fn new(p: Vec3) -> Self {
        Aabb { min: p, max: p }
    }

    /// Create a bounding box that covers the cube centered at `center` with a
    /// given `size`.
    pub fn cube(center: Vec3, size: f64) -> Self {
        Self::with_dimensions(center - size / 2.0, Vec3::replicate(size))
    }

    /// Create a bounding box that starts at th given min point with the given
    /// dimensions.
    pub fn with_dimensions(min: Vec3, dim: Vec3) -> Self {
        let mut aabb = Self::new(min);
        aabb.expand(min + dim);
        aabb
    }

    /// Build a bounding box that covers all the points in the given iterator.
    /// Returns `None` if there are no points to cover.
    pub fn from_points(it: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut it = it.into_iter();

        let p0 = it.next()?;
        let mut aabb = Aabb::new(p0);

        for v in it {
            aabb.expand(v);
        }

        Some(aabb)
    }

    /// Return the lowest point of the bounding box.
    pub fn min(&self) -> Vec3 {
        self.min
    }

    /// Return the highest point of the bounding box.
    pub fn max(&self) -> Vec3 {
        self.max
    }

    /// Return the center of the bounding box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) / 2.0
    }

    /// Return the dimensions of the bounding box.
    pub fn dimensions(&self) -> Vec3 {
        self.max - self.min
    }

    /// Expand the bounding box so that it covers the given point too.
    pub fn expand(&mut self, p: Vec3) {
        if p.x < self.min.x {
            self.min.x = p.x;
        } else if p.x > self.max.x {
            self.max.x = p.x;
        }

        if p.y < self.min.y {
            self.min.y = p.y;
        } else if p.y > self.max.y {
            self.max.y = p.y;
        }

        if p.z < self.min.z {
            self.min.z = p.z;
        } else if p.z > self.max.z {
            self.max.z = p.z;
        }
    }

    /// Consume the bounding box and return a new one that also covers the
    /// passed point.
    pub fn expanded(mut self, p: Vec3) -> Self {
        self.expand(p);
        self
    }

    /// Translate this `Aabb` by the given amount.
    pub fn translated(&self, p: Vec3) -> Self {
        Self {
            min: self.min + p,
            max: self.max + p,
        }
    }

    /// Expand the bounding box so that it covers another bounding box too.
    pub fn union(&self, aabb: &Aabb) -> Self {
        let mut out = self.clone();
        out.expand(aabb.min);
        out.expand(aabb.max);
        out
    }

    /// Expand the bounding box so that it covers another bounding box too.
    pub fn intersection(&self, aabb: &Aabb) -> Option<Self> {
        let b = Aabb {
            min: Vec3::new(
                self.min.x.max(aabb.min.x),
                self.min.y.max(aabb.min.y),
                self.min.z.max(aabb.min.z),
            ),
            max: Vec3::new(
                self.max.x.min(aabb.max.x),
                self.max.y.min(aabb.max.y),
                self.max.z.min(aabb.max.z),
            ),
        };

        if b.min.x > b.max.x || b.min.y > b.max.y || b.min.z > b.max.z {
            None
        } else {
            Some(b)
        }
    }

    /// Check if the bounding box contains the given point.
    pub fn contains(&self, pt: &Vec3) -> bool {
        self.min.x <= pt.x
            && self.max.x >= pt.x
            && self.min.y <= pt.y
            && self.max.y >= pt.y
            && self.min.z <= pt.z
            && self.max.z >= pt.z
    }

    /// Check whether a `Ray` intersects a `Aabb` and returns the t parameters
    /// of the first and last point of intersections. Note that a negative t
    /// means
    pub fn ray_intersection(&self, ray: &Ray) -> Option<(f64, f64)> {
        let max = self.max;
        let min = self.min;

        let mut tmin = (min.x - ray.origin.x) / ray.dir.x;
        let mut tmax = (max.x - ray.origin.x) / ray.dir.x;
        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (min.y - ray.origin.y) / ray.dir.y;
        let mut tymax = (max.y - ray.origin.y) / ray.dir.y;
        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if tmin > tymax || tymin > tmax {
            return None;
        }

        tmin = tmin.max(tymin);
        tmax = tmax.min(tymax);

        let mut tzmin = (min.z - ray.origin.z) / ray.dir.z;
        let mut tzmax = (max.z - ray.origin.z) / ray.dir.z;
        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if tmin > tzmax || tzmin > tmax {
            return None;
        }

        tmin = tmin.max(tzmin);
        tmax = tmax.min(tzmax);

        Some((tmin, tmax))
    }

    /// Get the bounding sphere of this `Aabb`.
    pub fn bounding_sphere(&self) -> (Vec3, f64) {
        let c = self.center();
        let r = self.min().dist(c);

        (c, r)
    }
}

impl std::iter::Extend<Vec3> for Aabb {
    fn extend<T: IntoIterator<Item = Vec3>>(&mut self, iter: T) {
        for v in iter {
            self.expand(v);
        }
    }
}

impl Transform for Aabb {
    fn transform(&self, m: &Mat4) -> Self {
        // http://dev.theomader.com/transform-bounding-boxes/
        let r = Vec3::new(m.data[0][0], m.data[1][0], m.data[2][0]);
        let u = Vec3::new(m.data[0][1], m.data[1][1], m.data[2][1]);
        let b = Vec3::new(m.data[0][2], m.data[1][2], m.data[2][2]);
        let t = Vec3::new(m.data[0][3], m.data[1][3], m.data[2][3]);

        let xa = r * self.min.x;
        let xb = r * self.max.x;
        let ya = u * self.min.y;
        let yb = u * self.max.y;
        let za = b * self.min.z;
        let zb = b * self.max.z;

        let Aabb { min: xa, max: xb } = Aabb::new(xa).expanded(xb);
        let Aabb { min: ya, max: yb } = Aabb::new(ya).expanded(yb);
        let Aabb { min: za, max: zb } = Aabb::new(za).expanded(zb);

        Aabb {
            min: xa + ya + za + t,
            max: xb + yb + zb + t,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut aabb = Aabb::new(Vec3::zero());

        assert_eq!(aabb.min(), Vec3::zero());
        assert_eq!(aabb.max(), Vec3::zero());
        assert_eq!(aabb.center(), Vec3::zero());

        aabb.expand(Vec3::new(-2.0, 0.0, 1.0));
        assert_eq!(aabb.min(), Vec3::new(-2.0, 0.0, 0.0),);
        assert_eq!(aabb.max(), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(aabb.center(), Vec3::new(-1.0, 0.0, 0.5));

        aabb.expand(Vec3::new(8.0, 8.0, -5.0));
        assert_eq!(aabb.min(), Vec3::new(-2.0, 0.0, -5.0));
        assert_eq!(aabb.max(), Vec3::new(8.0, 8.0, 1.0));
        assert_eq!(aabb.center(), Vec3::new(3.0, 4.0, -2.0));
    }

    #[test]
    fn test_cube() {
        let aabb = Aabb::cube(Vec3::new(0.0, 1.0, -1.0), 2.0);
        assert_eq!(
            aabb,
            Aabb {
                min: Vec3::new(-1.0, 0.0, -2.0),
                max: Vec3::new(1.0, 2.0, 0.0),
            }
        );
        assert_eq!(aabb.dimensions(), Vec3::replicate(2.0));
    }

    #[test]
    fn test_from_iter() {
        assert_eq!(Aabb::from_points(vec![]), None);

        assert_eq!(
            Aabb::from_points(vec![
                Vec3::zero(),
                Vec3::new(-2.0, 10.0, 2.0),
                Vec3::new(0.0, 1.0, -2.0)
            ]),
            Some(Aabb {
                min: Vec3::new(-2.0, 0.0, -2.0),
                max: Vec3::new(0.0, 10.0, 2.0)
            })
        );
    }

    #[test]
    fn test_union() {
        let mut aabb = Aabb::new(Vec3::zero());
        aabb = aabb.union(&Aabb::new(Vec3::new(1.0, 0.0, 2.0)));

        assert_eq!(
            aabb,
            Aabb {
                min: Vec3::zero(),
                max: Vec3::new(1.0, 0.0, 2.0)
            }
        );

        aabb = aabb.union(&Aabb::new(Vec3::new(-5.0, -1.0, -3.0)));
        assert_eq!(
            aabb,
            Aabb {
                min: Vec3::new(-5.0, -1.0, -3.0),
                max: Vec3::new(1.0, 0.0, 2.0)
            }
        );
    }

    #[test]
    fn test_intersection() {
        let mut aabb = Aabb::new(Vec3::zero());
        assert_eq!(aabb.intersection(&aabb), Some(aabb.clone()));
        aabb.expand(Vec3::new(10.0, 20.0, 10.0));

        assert_eq!(
            aabb.intersection(&Aabb::cube(Vec3::new(5.0, 10.0, 5.0), 6.0)),
            Some(Aabb {
                min: Vec3::new(2.0, 7.0, 2.0),
                max: Vec3::new(8.0, 13.0, 8.0),
            })
        );

        assert_eq!(
            aabb.intersection(&Aabb::cube(Vec3::new(-5.0, -5.0, -5.0), 20.0)),
            Some(Aabb {
                min: Vec3::zero(),
                max: Vec3::replicate(5.0),
            })
        );
    }

    #[test]
    fn test_dimensions() {
        assert_eq!(Aabb::new(Vec3::zero()).dimensions(), Vec3::zero());

        assert_eq!(
            Aabb::from_points(vec![
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::zero(),
                Vec3::new(-1.0, 0.0, 0.0)
            ])
            .unwrap()
            .dimensions(),
            Vec3::new(2.0, 2.0, 3.0)
        );
    }

    #[test]
    fn test_contains() {
        let aabb = Aabb::from_points(vec![Vec3::zero(), Vec3::new(-10.0, 2.0, 3.0)]).unwrap();

        assert!(aabb.contains(&Vec3::zero()));
        assert!(aabb.contains(&Vec3::new(-10.0, 2.0, 3.0)));
        assert!(aabb.contains(&Vec3::new(-8.0, 1.0, 2.0)));

        assert!(!aabb.contains(&Vec3::new(-20.0, 0.0, 0.0)));
        assert!(!aabb.contains(&Vec3::new(0.0, -5.0, 0.0)));
        assert!(!aabb.contains(&Vec3::new(0.0, 1.0, 4.0)));
    }

    #[test]
    fn test_ray_intersection() {
        let aabb = Aabb::from_points(vec![Vec3::zero(), Vec3::new(-10.0, 2.0, 3.0)]).unwrap();

        assert_eq!(
            aabb.ray_intersection(&Ray::new(Vec3::zero(), Vec3::new(-1.0, 0.0, 1.0))),
            Some((0.0, 3.0))
        );
        assert_eq!(
            aabb.ray_intersection(&Ray::new(
                Vec3::new(1.0, 1.0, 2.0),
                Vec3::new(-2.0, -1.0, 0.0)
            )),
            Some((0.5, 1.0))
        );
        assert_eq!(
            aabb.ray_intersection(&Ray::new(
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(-1.0, 0.0, 1.0)
            )),
            Some((1.0, 2.0))
        );

        assert_eq!(
            aabb.ray_intersection(&Ray::new(
                Vec3::new(1.0, 1.0, 1.0),
                Vec3::new(1.0, 0.0, 1.0)
            )),
            Some((-1.0, -1.0))
        );
        assert!(aabb
            .ray_intersection(&Ray::new(
                Vec3::new(-11.0, 6.0, 1.0),
                Vec3::new(-1.0, 0.0, 1.0)
            ))
            .is_none());

        let aabb = Aabb::from_points(vec![Vec3::new(-10.0, -1.0, -7.0), Vec3::new(0.0, 1.0, 1.0)])
            .unwrap();
        assert_eq!(
            aabb.ray_intersection(&Ray::new(
                Vec3::new(1.0, 0.0, 2.0),
                Vec3::new(-1.0, 1.0, -1.0)
            )),
            Some((1.0, 1.0))
        );

        let aabb = Aabb::from_points(vec![Vec3::new(-7.0, -1.0, -3.0), Vec3::zero()]).unwrap();
        assert_eq!(
            aabb.ray_intersection(&Ray::new(Vec3::zero(), Vec3::new(1.0, 1.0, 1.0))),
            Some((-1.0, 0.0))
        );
    }
}
