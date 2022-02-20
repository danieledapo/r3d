use std::iter;

use crate::Vec3;

/// A `Polyline` object represented by a series of points.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline {
    /// The points that make this `Polyline` object.
    pub points: Vec<Vec3>,
}

impl Polyline {
    /// Create a new empty `Polyline`.
    pub fn new() -> Self {
        Self { points: vec![] }
    }

    /// Return the number of points the `Polyline` is made of.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Return whether the the `Polyline` is empty or not.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Add a new point to the `Polyline`.
    pub fn push(&mut self, v: Vec3) {
        self.points.push(v);
    }

    /// Iterator over all the points.
    pub fn iter(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.points.iter().cloned()
    }

    /// Return the total length of the `Polyline`.
    pub fn norm(&self) -> f64 {
        self.points
            .iter()
            .zip(self.points.iter().skip(1))
            .map(|(p0, p1)| p0.dist(*p1))
            .sum()
    }

    /// Return a new `Polyline` where every point is `eps` distance from the
    /// previous point.
    pub fn chop(&self, eps: f64) -> Self {
        let mut out = Self::new();

        for (&s, &e) in self.points.iter().zip(self.points.iter().skip(1)) {
            let d = e - s;
            let dir = d.normalized();
            let maxl = d.norm();

            let mut l = 0.0;
            while l < maxl {
                out.push(s + dir * l);
                l += eps;
            }
        }

        out
    }

    /// Simplify the Polyline up to the given precision using the
    /// Ramer-Douglas-Peucker algorithm.
    pub fn simplified(mut self, eps: f64) -> Self {
        if self.len() < 3 {
            return self;
        }

        let a = self.points[0];
        let b = *self.points.last().unwrap();

        let mut index = 0;
        let mut maxd = 0.0;
        for (i, p) in self
            .points
            .iter()
            .enumerate()
            .take(self.points.len() - 2)
            .skip(1)
        {
            let d = p.segment_dist(a, b);
            if d > maxd {
                maxd = d;
                index = i;
            }
        }

        if maxd > eps {
            let right = self.points.split_off(index);
            self.points.push(right[0]);

            let mut path1 = self.simplified(eps);
            let path2 = Self::from(right).simplified(eps);

            path1.points.extend_from_slice(&path2.points[1..]);
            return path1;
        }

        Self::from(vec![a, b])
    }
}

impl Default for Polyline {
    fn default() -> Self {
        Self::new()
    }
}

impl std::convert::From<Vec<Vec3>> for Polyline {
    fn from(v: Vec<Vec3>) -> Self {
        Polyline { points: v }
    }
}

impl iter::FromIterator<Vec3> for Polyline {
    fn from_iter<T: IntoIterator<Item = Vec3>>(iter: T) -> Self {
        Polyline {
            points: iter.into_iter().collect(),
        }
    }
}

impl iter::Extend<Vec3> for Polyline {
    fn extend<T: IntoIterator<Item = Vec3>>(&mut self, iter: T) {
        self.points.extend(iter)
    }
}
