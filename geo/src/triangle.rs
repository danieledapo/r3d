use crate::vec3::Vec3;

/// Calculate the area of a triangle. If it is made up by 3
/// collinear points then the area is 0.
pub fn area(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> f64 {
    let e0 = *v1 - *v0;
    let e1 = *v2 - *v0;

    e0.cross(&e1).norm() / 2.0
}

/// Calculate the normal of a triangle.
pub fn normal(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Vec3 {
    let e0 = *v1 - *v0;
    let e1 = *v2 - *v0;

    e0.cross(&e1).normalized()
}

/// Calculate the [centroid][0] of a triangle.
///
/// [0]: https://en.wikipedia.org/wiki/Centroid
///
pub fn centroid(v0: &Vec3, v1: &Vec3, v2: &Vec3) -> Vec3 {
    (*v0 + *v1 + *v2) / 3.0
}

/// Compute the [barycentric coordinates][0] of a point `p` inside a triangle
/// and return them in a `Vec3`. Return `None` if `p` lies outside the triangle.
///
/// [0]: https://en.wikipedia.org/wiki/Barycentric_coordinate_system
///
pub fn barycentric((v0, v1, v2): (&Vec3, &Vec3, &Vec3), p: &Vec3) -> Option<Vec3> {
    let e0 = *v2 - *v0;
    let e1 = *v1 - *v0;

    let ep = *p - *v0;

    let dot00 = e0.dot(&e0);
    let dot01 = e0.dot(&e1);
    let dot11 = e1.dot(&e1);

    let den = dot00 * dot11 - dot01 * dot01;

    // collinear or degenerate triangle
    if den == 0.0 {
        return None;
    }

    let dot12 = e1.dot(&ep);
    let dot02 = e0.dot(&ep);

    let u = (dot11 * dot02 - dot01 * dot12) / den;
    let v = (dot00 * dot12 - dot01 * dot02) / den;

    // valid barycentric coordinates must always sum to 1 and each component
    // should be in [0, 1], if they do not then`p` is outside the triangle
    if u < 0.0 || u > 1.0 || v < 0.0 || v > 1.0 {
        None
    } else {
        Some(Vec3::new(1.0 - u - v, v, u))
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;
    use crate::triangle;

    #[test]
    fn test_triangle_area() {
        assert_eq!(
            triangle::area(
                &Vec3::new(0.0, 0.0, 0.0),
                &Vec3::new(50.0, 0.0, 0.0),
                &Vec3::new(25.0, 10.0, 0.0)
            ),
            250.0
        );

        assert_eq!(
            triangle::area(
                &Vec3::new(0.0, 0.0, 0.0),
                &Vec3::new(50.0, 0.0, 0.0),
                &Vec3::new(100.0, 0.0, 0.0)
            ),
            0.0
        );
    }

    #[test]
    fn test_triangle_normal() {
        assert_eq!(
            triangle::normal(
                &Vec3::new(2.0, 2.0, 2.0),
                &Vec3::new(10.0, 15.0, 2.0),
                &Vec3::new(4.0, 10.0, 2.0)
            ),
            Vec3::new(0.0, 0.0, 1.0)
        );

        assert_eq!(
            triangle::normal(
                &Vec3::new(2.0, 2.0, 2.0),
                &Vec3::new(10.0, 15.0, 2.0),
                &Vec3::new(4.0, 10.0, 2.0)
            ),
            Vec3::new(0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn test_triangle_centroid() {
        assert_eq!(
            triangle::centroid(
                &Vec3::new(2.0, 5.0, 2.0),
                &Vec3::new(10.0, 15.0, 2.0),
                &Vec3::new(6.0, 10.0, 2.0)
            ),
            Vec3::new(6.0, 10.0, 2.0)
        );
    }

    #[test]
    fn test_triangle_barycentric() {
        let v0 = Vec3::new(-20.0, -20.0, 0.0);
        let v1 = Vec3::new(0.0, 0.0, 0.0);
        let v2 = Vec3::new(-10.0, -2.0, 0.0);

        let tri = (&v0, &v1, &v2);

        // triangle vertices always have valid bary coords
        assert_eq!(
            triangle::barycentric(tri, &v0),
            Some(Vec3::new(1.0, 0.0, 0.0))
        );

        assert_eq!(
            triangle::barycentric(tri, &v1),
            Some(Vec3::new(0.0, 1.0, 0.0))
        );

        assert_eq!(
            triangle::barycentric(tri, &v2),
            Some(Vec3::new(0.0, 0.0, 1.0))
        );

        // random point inside has valid coords
        assert_eq!(
            triangle::barycentric(tri, &(v0 * 0.25 + v1 * 0.25 + v2 * 0.5)),
            Some(Vec3::new(0.25, 0.25, 0.5))
        );

        // outside point has not valid coords
        assert_eq!(triangle::barycentric(tri, &Vec3::new(10.0, 0.0, 0.0)), None);
        assert_eq!(
            triangle::barycentric(tri, &Vec3::new(-5.0, -10.0, 0.0)),
            None
        );

        // collinear triangle doesn't return valid bary coords
        assert_eq!(
            triangle::barycentric(
                (
                    &Vec3::new(10.0, 10.0, 10.0),
                    &Vec3::new(20.0, 20.0, 20.0),
                    &Vec3::new(0.0, 0.0, 0.0)
                ),
                &Vec3::new(10.0, 10.0, 10.0)
            ),
            None
        );
    }
}
