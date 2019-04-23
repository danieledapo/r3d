use crate::spatial_index::Shape;
use crate::util::ksmallest_by;
use crate::{Aabb, Axis, Vec3};

/// maximum number of elements each leaf can contain.
const LEAF_SIZE: usize = 8;

/// A [K-d tree][0] is a space partitioning data structure for organizing points
/// in a k-dimensional space. In our case, `KdTree` is actually a kdtree with
/// `k=3`.
///
/// A K-d tree is created from a set of `Shape`s that are recursively
/// partitioned according to the axis that best splits the center of the shapes
/// into two collections.
///
/// [0]: https://en.wikipedia.org/wiki/K-d_tree
#[derive(Debug, Clone, PartialEq)]
pub struct KdTree<T> {
    root: Node<T>,
}

#[derive(Debug, Clone, PartialEq)]
enum Node<T> {
    Leaf {
        data: Vec<T>,
    },
    Branch {
        left: Box<Node<T>>,
        right: Box<Node<T>>,
        split_value: f64,
        split_axis: Axis,
    },
}

impl<T> KdTree<T>
where
    T: Shape,
{
    /// Create a new `KdTree` that contains all the given shapes.
    pub fn new(shapes: Vec<T>) -> Self {
        let bboxes = shapes.iter().map(|s| s.bbox()).collect();

        KdTree {
            root: Node::new(shapes, bboxes),
        }
    }
}

impl<T> Node<T>
where
    T: Shape,
{
    fn new(shapes: Vec<T>, bboxes: Vec<Aabb>) -> Self {
        if shapes.len() <= LEAF_SIZE {
            return Node::Leaf { data: shapes };
        }

        let (split_axis, split_value) = best_partitioning(&bboxes);

        let (left, right) = partition(shapes, bboxes, split_axis, split_value);

        Node::Branch {
            left: Box::new(Node::new(left.0, left.1)),
            right: Box::new(Node::new(right.0, right.1)),

            split_value,
            split_axis,
        }
    }
}

/// Check if a `Aabb` semantically lies on the left of the given split_axis and split_value.
fn bbox_in_left(bbox: &Aabb, axis: Axis, c: f64) -> bool {
    (bbox.min()[axis] + bbox.max()[axis]) / 2.0 <= c
}

/// Find the best best partitioning (split_axis and split_value) for a given
/// collection of `Aabb` such that the shapes are well distributed over the
/// resulting two partitions.
fn best_partitioning(bboxes: &[Aabb]) -> (Axis, f64) {
    let mut bbox = bboxes[0].clone();

    let mut centers = Vec::with_capacity(bboxes.len());
    centers.push(bboxes[1].center());

    if bboxes.len() > 1 {
        for b in &bboxes[1..] {
            centers.push(b.center());
            bbox.union(&b);
        }
    }

    let Vec3 {
        x: range_x,
        y: range_y,
        z: range_z,
    } = bbox.dimensions();

    let axis = if range_x > range_y && range_x > range_z {
        Axis::X
    } else if range_y > range_x && range_y > range_z {
        Axis::Y
    } else {
        Axis::Z
    };

    let p = centers.len() / 2;
    let mid = *ksmallest_by(&mut centers, p, |a, b| {
        a[axis].partial_cmp(&b[axis]).unwrap()
    })
    .unwrap();

    (axis, mid[axis])
}

/// Partition the given `Shape`s and their `Aabb`s using the given `split_axis`
/// and `split_value`.
fn partition<T: Shape>(
    mut shapes: Vec<T>,
    mut bboxes: Vec<Aabb>,
    split_axis: Axis,
    split_value: f64,
) -> ((Vec<T>, Vec<Aabb>), (Vec<T>, Vec<Aabb>)) {
    let mut left = vec![];
    let mut left_bboxes = vec![];

    let mut right = vec![];
    let mut right_bboxes = vec![];

    while let Some(obj) = shapes.pop() {
        let bbox = bboxes.pop().unwrap();

        if bbox_in_left(&bbox, split_axis, split_value) {
            left.push(obj);
            left_bboxes.push(bbox);
        } else {
            right.push(obj);
            right_bboxes.push(bbox);
        }
    }

    ((left, left_bboxes), (right, right_bboxes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let kd = KdTree::new(vec![
            Vec3::zero(),
            Vec3::new(-1.0, 2.0, 0.0),
            Vec3::new(8.0, 6.0, -1.0),
        ]);

        assert_eq!(
            kd,
            KdTree {
                root: Node::Leaf {
                    data: vec![
                        Vec3::zero(),
                        Vec3::new(-1.0, 2.0, 0.0),
                        Vec3::new(8.0, 6.0, -1.0),
                    ]
                }
            }
        );

        let kd = KdTree::new(vec![
            Vec3::zero(),
            Vec3::new(-1.0, 2.0, 0.0),
            Vec3::new(8.0, 6.0, -1.0),
            Vec3::new(-1.0, -3.0, 2.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(10.0, 1.0, -4.0),
            Vec3::new(-9.0, -3.0, -3.0),
            Vec3::new(0.0, -6.0, 2.0),
            Vec3::new(-3.0, -3.0, 6.0),
        ]);

        assert_eq!(
            kd,
            KdTree {
                root: Node::Branch {
                    split_value: -1.0,
                    split_axis: Axis::X,

                    left: Box::new(Node::Leaf {
                        data: vec![
                            Vec3::new(-3.0, -3.0, 6.0),
                            Vec3::new(-9.0, -3.0, -3.0),
                            Vec3::new(-1.0, -3.0, 2.0),
                            Vec3::new(-1.0, 2.0, 0.0)
                        ]
                    }),
                    right: Box::new(Node::Leaf {
                        data: vec![
                            Vec3::new(0.0, -6.0, 2.0),
                            Vec3::new(10.0, 1.0, -4.0),
                            Vec3::new(0.0, 0.0, 1.0),
                            Vec3::new(8.0, 6.0, -1.0),
                            Vec3::new(0.0, 0.0, 0.0)
                        ]
                    }),
                }
            }
        );
    }

    #[test]
    fn test_best_partitioning() {
        assert_eq!(
            best_partitioning(&[
                Aabb::new(Vec3::zero()).expanded(&Vec3::new(10.0, 10.0, 10.0)),
                Aabb::new(Vec3::new(1.0, 2.0, 3.0)).expanded(&Vec3::new(7.0, 2.0, 7.0)),
                Aabb::new(Vec3::new(-1.0, -2.0, 3.0)).expanded(&Vec3::new(1.0, 1.0, 3.0)),
            ]),
            (Axis::Y, 2.0)
        );

        assert_eq!(
            best_partitioning(&[
                Aabb::new(Vec3::zero()).expanded(&Vec3::new(10.0, 10.0, 10.0)),
                Aabb::new(Vec3::new(1.0, 2.0, 3.0)).expanded(&Vec3::new(7.0, 2.0, 7.0)),
                Aabb::new(Vec3::new(-1.0, -2.0, 3.0)).expanded(&Vec3::new(1.0, 1.0, 3.0)),
                Aabb::new(Vec3::new(3.0, 3.0, 3.0)),
                Aabb::new(Vec3::new(-3.0, -5.0, -10.0)).expanded(&Vec3::new(-1.0, -3.0, 2.0)),
            ]),
            (Axis::Z, 3.0)
        );
    }

}
