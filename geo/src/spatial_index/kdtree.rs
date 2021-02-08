use std::ops::Deref;
use std::sync::Arc;

use crate::ray::Ray;
use crate::spatial_index::{Intersection, Shape};
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
        // Arc is needed because T might be shared between left and right if
        // T.bbox()[split_axis] contains the split_value
        data: Vec<Arc<T>>,
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
            root: Node::new(shapes.into_iter().map(Arc::new).collect(), bboxes),
        }
    }

    /// Find the intersection, if any, between the objects in the `KdTree` and a
    /// given `Ray`. The intersection is defined by the shape and its t
    /// parameter with respect to the ray.
    pub fn intersection(&self, ray: &Ray) -> Option<(&T, <T as Shape>::Intersection)> {
        self.intersections(ray).next()
    }

    /// Find all the intersections between the objects in the `KdTree` and a
    /// given ray. Each intersection is defined by the shape and its t parameter
    /// with respect to the ray. The intersections are sorted by their t
    /// parameter.
    pub fn intersections(
        &self,
        ray: &Ray,
    ) -> impl Iterator<Item = (&T, <T as Shape>::Intersection)> {
        self.root.intersections(ray, 0.0, std::f64::INFINITY)
    }
}

impl<T> Node<T>
where
    T: Shape,
{
    fn new(shapes: Vec<Arc<T>>, bboxes: Vec<Aabb>) -> Self {
        if shapes.len() <= LEAF_SIZE {
            return Node::Leaf { data: shapes };
        }

        match best_partitioning(&bboxes) {
            None => Node::Leaf { data: shapes },
            Some((split_axis, split_value)) => {
                let (left, right) = partition(shapes, bboxes, split_axis, split_value);

                Node::Branch {
                    left: Box::new(Node::new(left.0, left.1)),
                    right: Box::new(Node::new(right.0, right.1)),

                    split_value,
                    split_axis,
                }
            }
        }
    }

    fn intersections(
        &self,
        ray: &Ray,
        tmin: f64,
        tmax: f64,
    ) -> impl Iterator<Item = (&T, <T as Shape>::Intersection)> + '_ {
        let mut node_stack = vec![(self, tmin, tmax)];

        let mut current_intersections: Vec<(&T, <T as Shape>::Intersection)> =
            Vec::with_capacity(LEAF_SIZE);

        let ray = ray.clone();
        std::iter::from_fn(move || {
            loop {
                if let Some(r) = current_intersections.pop() {
                    return Some(r);
                }

                let (node, tmin, tmax) = node_stack.pop()?;

                match node {
                    Node::Leaf { data } => {
                        current_intersections.clear();
                        current_intersections.extend(data.iter().flat_map(|s| {
                            let intersection = s.intersection(&ray)?;
                            let t = intersection.t();

                            if tmin <= t && tmax >= t {
                                Some((s.deref(), intersection))
                            } else {
                                None
                            }
                        }));

                        // sort in reverse order so that we can pop the sorted
                        // elements quickly
                        current_intersections
                            .sort_by(|(_, t1), (_, t2)| t2.t().partial_cmp(&t1.t()).unwrap());
                    }
                    Node::Branch {
                        left,
                        right,
                        split_axis,
                        split_value,
                    } => {
                        // virtually split the ray into two, one from tmin to tsplit and
                        // another one from tsplit to tmax.
                        let tsplit = (split_value - ray.origin[*split_axis]) / ray.dir[*split_axis];

                        // if tsplit is not finite then therenot much sense in
                        // splitting the ray, just try to find the intersection in left
                        // and right with current tmin and tmax
                        if !tsplit.is_finite() {
                            node_stack.push((right, tmin, tmax));
                            node_stack.push((left, tmin, tmax));
                            continue;
                        }

                        let left_first = (ray.origin[*split_axis] < *split_value)
                            || (ray.origin[*split_axis] == *split_value
                                && ray.dir[*split_axis] <= 0.0);

                        let (first, second) = if left_first {
                            (&left, &right)
                        } else {
                            (&right, &left)
                        };

                        // if tsplit > tmax or tsplit < 0 then the ray does not span
                        // both first and second, but only first
                        if tsplit > tmax || tsplit <= 0.0 {
                            node_stack.push((first, tmin, tmax));
                            continue;
                        }

                        // when tsplit < tmin then the ray actually only spans the
                        // second node
                        if tsplit < tmin {
                            node_stack.push((second, tmin, tmax));
                            continue;
                        }

                        // in the general case find the intersection in the first node
                        // first and then in second. The result is simply the first
                        // intersection with the smaller t.
                        node_stack.push((second, tsplit, tmax));
                        node_stack.push((first, tmin, tsplit));
                    }
                }
            }
        })
    }
}

/// Check where the bounding box lies wrt to the given axis and value. In
/// particular, it returns:
/// - (true, false) when the bbox is completely to the left
/// - (false, true) when the bbox is completely to the right
/// - (true, true) when the value is inside the bbox
/// - (false, true) when there's no intersection
fn partition_bbox(bbox: &Aabb, axis: Axis, c: f64) -> (bool, bool) {
    (bbox.min()[axis] <= c, bbox.max()[axis] >= c)
}

/// Find the best best partitioning (split_axis and split_value) for a given
/// collection of `Aabb` such that the shapes are well distributed over the
/// resulting two partitions. If it wasn't able to find a suitable partitioning
/// then `None` is returned.
fn best_partitioning(bboxes: &[Aabb]) -> Option<(Axis, f64)> {
    // the idea here is to find the median X,Y,Z values for the centers which
    // partition the space almost equally by definition.
    //
    // However, itstill possible to have the same median value multiple times
    // which can result in a non ideal partitioning. To mitigate this issue,
    // iterate over all the median values and find the one that best partitions
    // the input.
    //

    let partition_size = |bboxes, axis, value| {
        let mut lefties = 0;
        let mut rightists = 0;

        for b in bboxes {
            let (l, r) = partition_bbox(b, axis, value);
            if l {
                lefties += 1;
            }

            if r {
                rightists += 1;
            }
        }

        // the higher the score is the more unbalanced the partitioning is
        lefties.max(rightists)
    };

    let mut centers = bboxes.iter().map(|b| b.center()).collect::<Vec<_>>();

    let (split_axis, split_value, partition_size) = [Axis::X, Axis::Y, Axis::Z]
        .iter()
        .map(|axis| {
            let p = centers.len() / 2;
            let (_, mid, _) =
                centers.select_nth_unstable_by(p, |a, b| a[*axis].partial_cmp(&b[*axis]).unwrap());

            let value = mid[*axis];

            (axis, value, partition_size(bboxes, *axis, value))
        })
        .min_by(|(_, _, s1), (_, _, s2)| s1.partial_cmp(s2).unwrap())
        .unwrap();

    // if the best partitioning we found is no better than having everything
    // flat, then do not return any partitioning
    if partition_size == bboxes.len() {
        None
    } else {
        Some((*split_axis, split_value))
    }
}

/// Partition the given `Shape`s and their `Aabb`s using the given `split_axis`
/// and `split_value`.
fn partition<T: Shape>(
    mut shapes: Vec<Arc<T>>,
    mut bboxes: Vec<Aabb>,
    split_axis: Axis,
    split_value: f64,
) -> ((Vec<Arc<T>>, Vec<Aabb>), (Vec<Arc<T>>, Vec<Aabb>)) {
    let mut left = vec![];
    let mut left_bboxes = vec![];

    let mut right = vec![];
    let mut right_bboxes = vec![];

    while let Some(obj) = shapes.pop() {
        let bbox = bboxes.pop().unwrap();

        let (l, r) = partition_bbox(&bbox, split_axis, split_value);

        if l {
            left.push(obj.clone());
            left_bboxes.push(bbox.clone());
        }

        if r {
            right.push(obj);
            right_bboxes.push(bbox);
        }
    }

    ((left, left_bboxes), (right, right_bboxes))
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    use crate::vec3;
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
                        Arc::new(Vec3::zero()),
                        Arc::new(Vec3::new(-1.0, 2.0, 0.0)),
                        Arc::new(Vec3::new(8.0, 6.0, -1.0)),
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
                    split_axis: Axis::Y,

                    left: Box::new(Node::Leaf {
                        data: vec![
                            Arc::new(Vec3::new(1.0, -3.0, 6.0)),
                            Arc::new(Vec3::new(-3.0, -3.0, 6.0)),
                            Arc::new(Vec3::new(0.0, -6.0, 2.0)),
                            Arc::new(Vec3::new(-9.0, -3.0, -3.0)),
                            Arc::new(Vec3::new(0.0, 0.0, 1.0)),
                            Arc::new(Vec3::new(-1.0, -3.0, 2.0)),
                            Arc::new(Vec3::new(0.0, 0.0, 0.0))
                        ]
                    }),
                    right: Box::new(Node::Leaf {
                        data: vec![
                            Arc::new(Vec3::new(0.0, 5.0, -1.0)),
                            Arc::new(Vec3::new(10.0, 1.0, -4.0)),
                            Arc::new(Vec3::new(0.0, 0.0, 1.0)),
                            Arc::new(Vec3::new(8.0, 6.0, -1.0)),
                            Arc::new(Vec3::new(-1.0, 2.0, 0.0)),
                            Arc::new(Vec3::new(0.0, 0.0, 0.0)),
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
                Aabb::new(Vec3::new(5.0, 0.0, 0.0)).expanded(&Vec3::new(10.0, 10.0, 10.0)),
                Aabb::new(Vec3::new(1.0, 2.0, 3.0)).expanded(&Vec3::new(7.0, 2.0, 7.0)),
                Aabb::new(Vec3::new(-1.0, -2.0, 3.0)).expanded(&Vec3::new(1.0, 1.0, 3.0)),
            ]),
            Some((Axis::X, 4.0))
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
            Some((Axis::Y, 2.0))
        );
    }

    proptest! {
        #[test]
        fn prop_kdtree_of_points_intersects_its_points_at_t0(
            pts in vec3::distinct_vec3(2..100)
        ) {
            let bbox = Aabb::from_iter(pts.iter().cloned()).unwrap();
            let center = bbox.center();

            let tree = KdTree::new(pts.clone());

            for p in pts {
                let d = (center - p).normalized();
                let ray = Ray::new(p, d);

                let int = tree.intersection(&ray);

                assert!(int.is_some(), "ray {:?}", ray);

                let (intersected, t) = int.unwrap();
                assert_eq!(*intersected, p);
                assert_eq!(t, 0.0);
            }
        }
    }

    proptest! {
        #[test]
        fn prop_kdtree_of_points_does_not_intersect_points_not_contained(
            mut pts in vec3::distinct_vec3(4..200)
        ) {
            let n = pts.len();
            let outside_pts = pts.split_off(n / 2);

            let tree = KdTree::new(pts.clone());

            for (i, p) in outside_pts.iter().enumerate() {
                let ray = Ray::new(
                    *p,
                    (outside_pts[(i + 1) % outside_pts.len()] - *p).normalized(),
                );

                if let Some((v, _)) = tree.intersection(&ray) {
                    assert_ne!(v, p);
                }
            }
        }
    }
    proptest! {
        #[test]
        fn prop_kdtree_intersections_are_sorted(
            rpts in vec3::distinct_vec3(2..=2),
            ts in proptest::collection::hash_set(any::<i32>(), 2..50)
        ) {
            // generate points on ray so that we're sure we have multiple
            // intersections per ray

            let ray = Ray::new(rpts[0], rpts[1] - rpts[0]);

            let pts = ts.into_iter().map(|t| ray.point_at(f64::from(t))).collect::<Vec<_>>();
            let tree = KdTree::new(pts.clone());

            let ints = tree.intersections(&ray).collect::<Vec<_>>();

            for w in ints.windows(2) {
                assert!(w[0].1 <= w[1].1);
            }
        }
    }

    #[test]
    fn test_intersection_with_stall_ray() {
        let tree = KdTree::new(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 4.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(1.0, 0.0, 1.0),
        ]);

        match tree.root {
            Node::Branch {
                split_axis,
                split_value,
                ..
            } if split_axis == Axis::Z && split_value == 1.0 => {}
            _ => {
                unreachable!(
                    "this test does not make sense anymore, itmeant to test what happens when tsplit is not finite"
                );
            }
        }

        let ray = Ray::new(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(
            tree.intersection(&ray),
            Some((&Vec3::new(0.0, 0.0, 1.0), 0.0))
        );
    }

    #[test]
    fn test_best_partitioning_no_better() {
        let bboxes = vec![
            Aabb::new(Vec3::new(-0.1640625, -0.6953125, -0.9453125))
                .expanded(&Vec3::new(0.0, -0.6328125, -0.8828125)),
            Aabb::new(Vec3::new(-0.1640625, -0.7109375, -0.9296875))
                .expanded(&Vec3::new(-0.0625, -0.6328125, -0.8359375)),
            Aabb::new(Vec3::new(-0.234375, -0.7109375, -0.9296875))
                .expanded(&Vec3::new(-0.1171875, -0.6328125, -0.8359375)),
            Aabb::new(Vec3::new(-0.234375, -0.734375, -0.9140625))
                .expanded(&Vec3::new(-0.109375, -0.6328125, -0.71875)),
            Aabb::new(Vec3::new(-0.265625, -0.734375, -0.9140625))
                .expanded(&Vec3::new(-0.109375, -0.6328125, -0.71875)),
            Aabb::new(Vec3::new(-0.265625, -0.734375, -0.8203125))
                .expanded(&Vec3::new(-0.109375, -0.6640625, -0.703125)),
            Aabb::new(Vec3::new(-0.1171875, -0.734375, -0.8359375))
                .expanded(&Vec3::new(-0.09375, -0.7109375, -0.71875)),
            Aabb::new(Vec3::new(-0.1171875, -0.7265625, -0.8359375))
                .expanded(&Vec3::new(-0.09375, -0.7109375, -0.7421875)),
            Aabb::new(Vec3::new(-0.1171875, -0.7109375, -0.8828125))
                .expanded(&Vec3::new(-0.0625, -0.6953125, -0.8203125)),
        ];

        assert!(best_partitioning(&bboxes).is_none());
    }
}
