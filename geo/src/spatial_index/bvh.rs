use std::iter::FromIterator;

use crate::ray::Ray;
use crate::spatial_index::{Intersection, Shape};
use crate::{Aabb, Axis, Vec3};

/// A [Bounding volume hierarchy][0] is a tree data structure for collecting a
/// set of shapes that allows for quick intersection checking by pruning the
/// tree by looking at the `Aabb` of each shape.
///
/// The current implementation partitions the shapes by recursively splitting
/// the shapes by the axis that has the biggest change in coordinates between
/// all the centers of the bounding boxes.
///
/// [0]: https://en.wikipedia.org/wiki/Bounding_volume_hierarchy
#[derive(Debug, Clone, PartialEq)]
pub struct Bvh<T> {
    root: Option<Node<T>>,
    infinite_objects: Vec<T>,
}

#[derive(Debug, Clone, PartialEq)]
enum Node<T> {
    Branch {
        bbox: Aabb,
        left: Box<Node<T>>,
        right: Box<Node<T>>,
    },
    Leaf {
        data: T,
    },
}

impl<T> Bvh<T>
where
    T: Shape,
{
    /// Return the boundinx box for all the elements contained in the `Bvh`.
    /// Return `None` if empty.
    pub fn bbox(&self) -> Option<Aabb> {
        let nodes_bbox = self.root.as_ref().map(|n| match n {
            Node::Branch { bbox, .. } => bbox.clone(),
            Node::Leaf { data } => data.bbox(),
        });

        let infinite_bbox = if self.infinite_objects.is_empty() {
            None
        } else {
            let mut aabb = self.infinite_objects[0].bbox();
            for o in &self.infinite_objects[1..] {
                aabb = aabb.union(&o.bbox());
            }
            Some(aabb)
        };

        match (nodes_bbox, infinite_bbox) {
            (Some(nodes_bbox), Some(infinite_bbox)) => Some(nodes_bbox.union(&infinite_bbox)),
            (Some(bbox), None) | (None, Some(bbox)) => Some(bbox),
            (None, None) => None,
        }
    }

    /// Iterator over all the elements.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let mut stack = vec![];
        if let Some(n) = self.root.as_ref() {
            stack.push(n);
        }

        std::iter::from_fn(move || {
            while let Some(n) = stack.pop() {
                match n {
                    Node::Leaf { data } => return Some(data),
                    Node::Branch { left, right, .. } => {
                        stack.push(right);
                        stack.push(left);
                    }
                }
            }

            None
        })
        .chain(&self.infinite_objects)
    }

    /// Returns all the objects that intersect the given `Aabb`.
    pub fn bbox_intersections(&self, aabb: Aabb) -> impl Iterator<Item = &T> {
        let mut stack = vec![];
        if let Some(n) = self.root.as_ref() {
            stack.push(n);
        }

        {
            let aabb = aabb.clone();
            std::iter::from_fn(move || {
                while let Some(n) = stack.pop() {
                    match n {
                        Node::Leaf { data } => {
                            if data.bbox().intersection(&aabb).is_none() {
                                continue;
                            }
                            return Some(data);
                        }
                        Node::Branch { left, right, bbox } => {
                            if bbox.intersection(&aabb).is_some() {
                                stack.push(right);
                                stack.push(left);
                            }
                        }
                    }
                }

                None
            })
        }
        .chain(
            self.infinite_objects
                .iter()
                .filter(move |obj| obj.bbox().intersection(&aabb).is_some()),
        )
    }

    /// Return all the objects that intersect the given ray along with their t
    /// parameter.
    pub fn intersections(&self, ray: &Ray) -> impl Iterator<Item = (&T, T::Intersection)> {
        let mut stack = vec![];
        if let Some(n) = self.root.as_ref() {
            stack.push(n);
        }

        let ray = ray.clone();
        Intersections {
            stack,
            ray: ray.clone(),
        }
        .chain(self.infinite_objects.iter().filter_map(move |obj| {
            let inter = obj.intersection(&ray)?;
            if inter.t() >= 0.0 {
                Some((obj, inter))
            } else {
                None
            }
        }))
    }
}

pub struct Intersections<'s, T> {
    stack: Vec<&'s Node<T>>,
    ray: Ray,
}

impl<'s, T> std::iter::Iterator for Intersections<'s, T>
where
    T: Shape,
{
    type Item = (&'s T, T::Intersection);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(n) = self.stack.pop() {
            match n {
                Node::Leaf { data } => {
                    if let Some(inter) = data.intersection(&self.ray) {
                        if inter.t() >= 0.0 {
                            return Some((data, inter));
                        }
                    }
                }

                Node::Branch { bbox, left, right } => match bbox.ray_intersection(&self.ray) {
                    Some((t1, t2)) if t1 <= t2 && t2 >= 0.0 => {
                        self.stack.push(right);
                        self.stack.push(left);
                    }
                    _ => {}
                },
            }
        }

        None
    }
}

impl<'s, T: 's> FromIterator<T> for Bvh<T>
where
    T: Shape,
{
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
        let mut elems = vec![];
        let mut infinite_objects = vec![];

        for elem in it {
            let bbox = elem.bbox();
            if bbox.min().is_finite() && bbox.max().is_finite() {
                elems.push((elem, bbox));
            } else {
                infinite_objects.push(elem);
            }
        }

        Bvh {
            infinite_objects,
            root: if elems.is_empty() {
                None
            } else {
                Some(Node::new(elems))
            },
        }
    }
}

impl<T> Node<T>
where
    T: Shape,
{
    fn new(mut elems: Vec<(T, Aabb)>) -> Self {
        assert!(!elems.is_empty());

        if elems.len() == 1 {
            return Node::Leaf {
                data: elems.pop().unwrap().0,
            };
        }

        let (partition_axis, bbox) = Node::preprocess_elems(&elems);

        let pivot = elems.len() / 2;

        // select_nth actually partitions the elems so that bboxes before pivot
        // have a smaller dimensions than the median dimension and bboxes after
        // pivot have a greater dimension.
        elems.select_nth_unstable_by(pivot, |(_, b1), (_, b2)| {
            let c1 = b1.center()[partition_axis];
            let c2 = b2.center()[partition_axis];

            c1.total_cmp(&c2)
        });

        let right_elems = elems.split_off(pivot);

        Node::Branch {
            bbox,
            left: Box::new(Node::new(elems)),
            right: Box::new(Node::new(right_elems)),
        }
    }

    /// Gather some information about the input shapes and return the axis that
    /// has the biggest change in coordinates among all the shapes and the
    /// bounding box of the shapes.
    fn preprocess_elems(elems: &[(T, Aabb)]) -> (Axis, Aabb) {
        let mut bbox = elems[0].1.clone();
        let mut ranges = bbox.clone();

        if elems.len() > 1 {
            for (_, b) in &elems[1..] {
                ranges.expand(b.center());
                bbox = bbox.union(b);
            }
        }

        let Vec3 {
            x: range_x,
            y: range_y,
            z: range_z,
        } = ranges.dimensions();

        let axis = if range_x > range_y && range_x > range_z {
            Axis::X
        } else if range_y > range_x && range_y > range_z {
            Axis::Y
        } else {
            Axis::Z
        };

        (axis, bbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bvh_build() {
        let bvh: Bvh<Vec3> = vec![
            Vec3::new(10.0, -1.0, 7.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(8.0, 1.0, 4.0),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            bvh,
            Bvh {
                infinite_objects: vec![],
                root: Some(Node::Branch {
                    bbox: Aabb::from_points(vec![
                        Vec3::new(0.0, -1.0, 0.0),
                        Vec3::new(10.0, 2.0, 7.0)
                    ])
                    .unwrap(),

                    left: Box::new(Node::Leaf {
                        data: Vec3::new(0.0, 2.0, 0.0)
                    }),

                    right: Box::new(Node::Branch {
                        bbox: Aabb::from_points(vec![
                            Vec3::new(8.0, -1.0, 4.0),
                            Vec3::new(10.0, 1.0, 7.0)
                        ])
                        .unwrap(),

                        left: Box::new(Node::Leaf {
                            data: Vec3::new(8.0, 1.0, 4.0)
                        }),

                        right: Box::new(Node::Leaf {
                            data: Vec3::new(10.0, -1.0, 7.0)
                        })
                    })
                })
            }
        );

        assert_eq!(
            bvh.iter().collect::<Vec<_>>(),
            vec![
                &Vec3::new(0.0, 2.0, 0.0),
                &Vec3::new(8.0, 1.0, 4.0),
                &Vec3::new(10.0, -1.0, 7.0)
            ]
        );
    }

    #[test]
    fn test_intersections() {
        let bvh: Bvh<Vec3> = vec![
            Vec3::new(-10.0, -1.0, -7.0),
            Vec3::zero(),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(10.0, 1.0, 7.0),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            bvh.intersections(&Ray::new(Vec3::zero(), Vec3::new(1.0, 1.0, 1.0)))
                .collect::<Vec<_>>(),
            vec![(&Vec3::zero(), 0.0)]
        );

        assert_eq!(
            bvh.intersections(&Ray::new(
                Vec3::new(1.0, 0.0, 2.0),
                Vec3::new(-1.0, 1.0, -1.0)
            ))
            .collect::<Vec<_>>(),
            vec![(&Vec3::new(0.0, 1.0, 1.0), 1.0)]
        );

        assert_eq!(
            bvh.intersections(&Ray::new(Vec3::zero(), Vec3::new(0.0, 1.0, 1.0)))
                .collect::<Vec<_>>(),
            vec![(&Vec3::zero(), 0.0), (&Vec3::new(0.0, 1.0, 1.0), 1.0)]
        );

        assert_eq!(
            bvh.intersections(&Ray::new(
                Vec3::new(-11.0, -20.0, 100.0),
                Vec3::new(-1.0, -1.0, 1.0)
            ))
            .collect::<Vec<_>>(),
            vec![]
        );
    }
}
