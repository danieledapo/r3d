use std::cmp::Ordering;
use std::f64::EPSILON;
use std::iter::FromIterator;

use crate::aabb::Aabb;
use crate::util::ksmallest_by;
use crate::vec3::Vec3;
use crate::Axis;

pub trait Elem: std::fmt::Debug {
    fn bbox(&self) -> Aabb;
}

impl Elem for Vec3 {
    fn bbox(&self) -> Aabb {
        Aabb::new(*self)
    }
}

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
}

// TODO: it might be more efficient to store a Vec<T> in leaves because jumps in
// the heap can have more overhead? This could make construction faster too...
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

impl<T> FromIterator<T> for Bvh<T>
where
    T: Elem,
{
    fn from_iter<I: IntoIterator<Item = T>>(it: I) -> Self {
        let elems = it
            .into_iter()
            .map(|e| {
                let b = e.bbox();
                (e, b)
            })
            .collect::<Vec<_>>();

        Bvh {
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
    T: Elem,
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

        // ksmallest actually partitions the elems so that bboxes before pivot
        // have a smaller dimensions than the median dimension and bboxes after
        // pivot have a greater dimension.
        ksmallest_by(&mut elems, pivot, |(_, b1), (_, b2)| {
            let c1 = b1.center()[partition_axis];
            let c2 = b2.center()[partition_axis];

            if (c1 - c2).abs() < EPSILON {
                return Ordering::Equal;
            }

            if c1 < c2 {
                return Ordering::Less;
            }

            Ordering::Greater
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
                ranges.expand(&b.center());
                bbox.union(&b);
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
                root: Some(Node::Branch {
                    bbox: Aabb::from_iter(vec![
                        Vec3::new(0.0, -1.0, 0.0),
                        Vec3::new(10.0, 2.0, 7.0)
                    ])
                    .unwrap(),

                    left: Box::new(Node::Leaf {
                        data: Vec3::new(0.0, 2.0, 0.0)
                    }),

                    right: Box::new(Node::Branch {
                        bbox: Aabb::from_iter(vec![
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
    }
}
