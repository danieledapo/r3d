use crate::vec3::Vec3;

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

    /// Build a bounding box that covers all the points in the given iterator.
    /// Returns `None` if there are no points to cover.
    pub fn from_iter(it: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut it = it.into_iter();

        let p0 = it.next()?;
        let mut aabb = Aabb::new(p0);

        for v in it {
            aabb.expand(&v);
        }

        Some(aabb)
    }

    /// Return the lowest point of the bounding box.
    pub fn min(&self) -> &Vec3 {
        &self.min
    }

    /// Return the highest point of the bounding box.
    pub fn max(&self) -> &Vec3 {
        &self.max
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
    pub fn expand(&mut self, p: &Vec3) {
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

    /// Expand the bounding box so that it covers another bounding box too.
    pub fn union(&mut self, aabb: &Aabb) {
        self.expand(&aabb.min);
        self.expand(&aabb.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut aabb = Aabb::new(Vec3::zero());

        assert_eq!(aabb.min(), &Vec3::zero());
        assert_eq!(aabb.max(), &Vec3::zero());
        assert_eq!(aabb.center(), Vec3::zero());

        aabb.expand(&Vec3::new(-2.0, 0.0, 1.0));
        assert_eq!(aabb.min(), &Vec3::new(-2.0, 0.0, 0.0),);
        assert_eq!(aabb.max(), &Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(aabb.center(), Vec3::new(-1.0, 0.0, 0.5));

        aabb.expand(&Vec3::new(8.0, 8.0, -5.0));
        assert_eq!(aabb.min(), &Vec3::new(-2.0, 0.0, -5.0));
        assert_eq!(aabb.max(), &Vec3::new(8.0, 8.0, 1.0));
        assert_eq!(aabb.center(), Vec3::new(3.0, 4.0, -2.0));
    }

    #[test]
    fn test_from_iter() {
        assert_eq!(Aabb::from_iter(vec![]), None);

        assert_eq!(
            Aabb::from_iter(vec![
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
        aabb.union(&Aabb::new(Vec3::new(1.0, 0.0, 2.0)));

        assert_eq!(
            aabb,
            Aabb {
                min: Vec3::zero(),
                max: Vec3::new(1.0, 0.0, 2.0)
            }
        );

        aabb.union(&Aabb::new(Vec3::new(-5.0, -1.0, -3.0)));
        assert_eq!(
            aabb,
            Aabb {
                min: Vec3::new(-5.0, -1.0, -3.0),
                max: Vec3::new(1.0, 0.0, 2.0)
            }
        );
    }

    #[test]
    fn test_dimensions() {
        assert_eq!(Aabb::new(Vec3::zero()).dimensions(), Vec3::zero());

        assert_eq!(
            Aabb::from_iter(vec![
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::zero(),
                Vec3::new(-1.0, 0.0, 0.0)
            ])
            .unwrap()
            .dimensions(),
            Vec3::new(2.0, 2.0, 3.0)
        );
    }
}
