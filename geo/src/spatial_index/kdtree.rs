use crate::ray::Ray;
use crate::spatial_index::Shape;
use crate::util::ksmallest_by;
use crate::{Aabb, Axis};

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

    /// Find the intersection, if any, between the objects in the `KdTree` and a
    /// given `Ray`. The parameter
    pub fn intersection<'s>(&'s self, ray: &Ray) -> Option<(&'s T, f64)> {
        self.root.intersection(ray, 0.0, std::f64::INFINITY)
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

    fn intersection<'s>(&'s self, ray: &Ray, tmin: f64, tmax: f64) -> Option<(&'s T, f64)> {
        match self {
            Node::Leaf { data } => data
                .iter()
                .flat_map(|s| s.intersection(ray).map(|t| (s, t)))
                // .filter(|(_, t)| tmin <= *t && tmax >= *t)
                .min_by(|(_, t1), (_, t2)| t1.partial_cmp(t2).unwrap()),

            Node::Branch {
                left,
                right,
                split_axis,
                split_value,
            } => {
                // virtually split the ray into two, one from tmin to tsplit and
                // another one from tsplit to tmax.
                let tsplit = (split_value - ray.origin[*split_axis]) / ray.dir[*split_axis];

                let left_first = (ray.origin[*split_axis] < *split_value)
                    || (ray.origin[*split_axis] == *split_value && ray.dir[*split_axis] <= 0.0);

                let (first, second) = if left_first {
                    (&left, &right)
                } else {
                    (&right, &left)
                };

                // if tsplit > tmax or tsplit < 0 then the ray does not span
                // both first and second, but only first
                if tsplit > tmax || tsplit <= 0.0 {
                    return first.intersection(ray, tmin, tmax);
                }

                // when tsplit < tmin then the ray actually only spans the
                // second node
                if tsplit < tmin {
                    return second.intersection(ray, tmin, tmax);
                }

                // in the general case find the intersection in the first node
                // first and then in second. The result is simply the first
                // intersection with the smaller t.
                let i1 = first.intersection(ray, tmin, tsplit);
                if let Some((o, t)) = i1 {
                    if t < tsplit {
                        return Some((o, t));
                    }
                }

                let i2 = second.intersection(ray, tsplit, tmax);
                match (i1, i2) {
                    (None, None) => None,
                    (Some(i), None) | (None, Some(i)) => Some(i),
                    (Some((s1, t1)), Some((s2, t2))) => {
                        if t1 < t2 {
                            Some((s1, t1))
                        } else {
                            Some((s2, t2))
                        }
                    }
                }
            }
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
    // the idea here is to find the median X,Y,Z values for the centers which
    // partition the space almost equally by definition.
    //
    // However, it's still possible to have the same median value multiple times
    // which can result in a non ideal partitioning. To mitigate this issue,
    // iterate over all the median values and find the one that best partitions
    // the input.
    //

    let mut centers = bboxes.iter().map(|b| b.center()).collect::<Vec<_>>();

    let (split_axis, split_value, _) = [Axis::X, Axis::Y, Axis::Z]
        .iter()
        .map(|axis| {
            let p = centers.len() / 2;
            let mid = *ksmallest_by(&mut centers, p, |a, b| {
                a[*axis].partial_cmp(&b[*axis]).unwrap()
            })
            .unwrap();

            let value = mid[*axis];

            let lefties = bboxes
                .iter()
                .filter(|b| bbox_in_left(b, *axis, value))
                .count();

            // the higher the score is the more unbalanced the partitioning is
            let score = lefties.max(centers.len() - lefties);

            (axis, value, score)
        })
        .min_by(|(_, _, s1), (_, _, s2)| s1.partial_cmp(s2).unwrap())
        .unwrap();

    (*split_axis, split_value)
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

    use crate::Vec3;

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
            Vec3::new(0.0, 5.0, -1.0),
            Vec3::new(1.0, -3.0, 6.0),
        ]);

        assert_eq!(
            kd,
            KdTree {
                root: Node::Branch {
                    split_value: 0.0,
                    split_axis: Axis::Z,

                    left: Box::new(Node::Leaf {
                        data: vec![
                            Vec3::new(0.0, 5.0, -1.0),
                            Vec3::new(-9.0, -3.0, -3.0),
                            Vec3::new(10.0, 1.0, -4.0),
                            Vec3::new(8.0, 6.0, -1.0),
                            Vec3::new(-1.0, 2.0, 0.0),
                            Vec3::new(0.0, 0.0, 0.0)
                        ]
                    }),
                    right: Box::new(Node::Leaf {
                        data: vec![
                            Vec3::new(1.0, -3.0, 6.0),
                            Vec3::new(-3.0, -3.0, 6.0),
                            Vec3::new(0.0, -6.0, 2.0),
                            Vec3::new(0.0, 0.0, 1.0),
                            Vec3::new(-1.0, -3.0, 2.0)
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
            (Axis::X, 4.0)
        );

        assert_eq!(
            best_partitioning(&[
                Aabb::new(Vec3::new(-2.0, -1.0, 0.0)),
                Aabb::new(Vec3::zero()),
                Aabb::new(Vec3::new(3.0, 1.0, 2.0)),
                Aabb::new(Vec3::new(3.0, 2.0, 2.0)),
                Aabb::new(Vec3::new(3.0, 3.0, 2.0)),
                Aabb::new(Vec3::new(4.0, 4.0, 2.0)),
                Aabb::new(Vec3::new(5.0, 5.0, 2.0)),
            ]),
            (Axis::Y, 2.0)
        );
    }

}
