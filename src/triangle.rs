use std::cell::OnceCell;

use crate::vertex::Vertex;


pub struct Triangle<'a> {
    pub a: &'a Vertex,
    pub b: &'a Vertex,
    pub c: &'a Vertex,
    double_area: OnceCell<i32>,
}


impl<'a> Triangle<'a> {
    pub fn new(a: &'a Vertex, b: &'a Vertex, c: &'a Vertex) -> Triangle<'a> {
        Triangle { a: a, b: b, c: c, double_area: OnceCell::new() }
    }

    pub fn double_area(&self) -> i32 {
        *self.double_area.get_or_init(|| {
            let t1 = (self.b.x - self.a.x) * (self.c.y - self.a.y);
            let t2 = (self.c.x - self.a.x) * (self.b.y - self.a.y);
            t1 - t2
        })
    }

    pub fn area_sign(&self) -> i32 {
        self.double_area().signum()
    }

    pub fn has_collinear_points(&self) -> bool {
        self.area_sign() == 0
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn test_area_right_triangle() {
        let a = Vertex::new(0, 0);
        let b = Vertex::new(3, 0);
        let c = Vertex::new(0, 4);
        let triangle = Triangle::new(&a, &b, &c);
        let double_area = triangle.double_area();
        assert_eq!(double_area, 12);
    }

    // TODO want some better unit tests for the triangle area

    #[test]
    fn test_area_sign_clockwise() {
        let a = Vertex::new(0, 0);
        let b = Vertex::new(4, 3);
        let c = Vertex::new(1, 3);
        
        let cw = vec![
            Triangle::new(&a, &c, &b),
            Triangle::new(&c, &b, &a),
            Triangle::new(&b, &a, &c),
        ];
        for triangle in cw {
            assert_eq!(triangle.area_sign(), -1);
        }
    }

    #[test]
    fn test_area_sign_counter_clockwise() {
        let a = Vertex::new(0, 0);
        let b = Vertex::new(4, 3);
        let c = Vertex::new(1, 3);

        let ccw = vec![
            Triangle::new(&a, &b, &c),
            Triangle::new(&b, &c, &a),
            Triangle::new(&c, &a, &b),
        ];
        for triangle in ccw {
            assert_eq!(triangle.area_sign(), 1);
        }
    }

    #[test]
    fn test_area_sign_collinear() {
        let a = Vertex::new(0, 0);
        let b = Vertex::new(4, 3);
        let c = Vertex::new(1, 3);

        // This is choice with replacement over a 3-tuple, so there are
        // 3 * 3 * 3 = 27 total options and this generates all of them.
        let all_combos = std::iter::repeat(vec![&a, &b, &c].into_iter())
            .take(3)
            .multi_cartesian_product();
        
        for vertices in all_combos {
            let p0 = vertices[0];
            let p1 = vertices[1];
            let p2 = vertices[2];
            let triangle = Triangle::new(p0, p1, p2);
            
            if p0 == p1 || p0 == p2 || p1 == p2 {
                // If there's duplicate vertices, they should be detected
                // as collinear (zero area)
                assert!(triangle.has_collinear_points());
            } else {
                // If all vertices are unique then they're either clockwise
                // (negative area) or counter-clockwise (positive area)
                assert!(!triangle.has_collinear_points());
            }
        }
    }
}
