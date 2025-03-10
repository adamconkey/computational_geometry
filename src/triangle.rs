use std::cell::OnceCell;

use crate::{
    line_segment::LineSegment, point::Point, vertex::Vertex
};


pub struct Triangle<'a> {
    pub p1: &'a Point,
    pub p2: &'a Point,
    pub p3: &'a Point,
    area: OnceCell<f64>,
}

impl<'a> Triangle<'a> {
    pub fn new(p1: &'a Point, p2: &'a Point, p3: &'a Point) -> Triangle<'a> {
        Triangle { p1, p2, p3, area: OnceCell::new() }
    }

    pub fn from_vertices(v1: &'a Vertex, v2: &'a Vertex, v3: &'a Vertex) -> Triangle<'a> {
        Triangle::new(&v1.coords, &v2.coords, &v3.coords)
    }

    pub fn to_line_segments(&self) -> Vec<LineSegment> {
        let ls1 = LineSegment::new(self.p1, self.p2);
        let ls2 = LineSegment::new(self.p2, self.p3);
        let ls3 = LineSegment::new(self.p3, self.p1);
        vec![ls1, ls2, ls3]
    }

    pub fn area(&self) -> f64 {
        *self.area.get_or_init(|| {
            let t1 = (self.p2.x - self.p1.x) * (self.p3.y - self.p1.y);
            let t2 = (self.p3.x - self.p1.x) * (self.p2.y - self.p1.y);
            0.5 * (t1 - t2)
        })
    }

    pub fn has_collinear_points(&self) -> bool {
        self.area() == 0.0
    }

    pub fn contains(&self, p: Point) -> bool {
        if self.has_collinear_points() {
            return false;
        }
        for ls in self.to_line_segments() {
            if !p.left_on(&ls) {
                return false;
            }
        }
        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use crate::vertex::VertexId;

    #[test]
    fn test_from_vertices() {
        let id1 = VertexId::from(1u32);
        let id2 = VertexId::from(2u32);
        let id3 = VertexId::from(3u32);
        let v1 = Vertex::new(Point::new(0.0, 0.0), id1, id3, id2);
        let v2 = Vertex::new(Point::new(3.0, 0.0), id2, id1, id3);
        let v3 = Vertex::new(Point::new(0.0, 4.0), id3, id2, id1);
        let triangle = Triangle::from_vertices(&v1, &v2, &v3);
        assert_eq!(Point::new(0.0, 0.0), *triangle.p1);   
        assert_eq!(Point::new(3.0, 0.0), *triangle.p2);   
        assert_eq!(Point::new(0.0, 4.0), *triangle.p3);   
    }

    #[test]
    fn test_area_right_triangle() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 0.0);
        let c = Point::new(0.0, 4.0);
        let triangle = Triangle::new(&a, &b, &c);
        let area = triangle.area();
        assert_eq!(area, 6.0);
    }

    // TODO want some better unit tests for the triangle area

    #[test]
    fn test_area_clockwise() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(4.0, 3.0);
        let c = Point::new(1.0, 3.0);
        
        let cw = vec![
            Triangle::new(&a, &c, &b),
            Triangle::new(&c, &b, &a),
            Triangle::new(&b, &a, &c),
        ];
        for triangle in cw {
            assert!(triangle.area() < 0.0);
        }
    }

    #[test]
    fn test_area_counter_clockwise() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(4.0, 3.0);
        let c = Point::new(1.0, 3.0);

        let ccw = vec![
            Triangle::new(&a, &b, &c),
            Triangle::new(&b, &c, &a),
            Triangle::new(&c, &a, &b),
        ];
        for triangle in ccw {
            assert!(triangle.area() > 0.0);
        }
    }

    #[test]
    fn test_area_collinear() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(4.0, 3.0);
        let c = Point::new(1.0, 3.0);

        // This is choice with replacement over a 3-tuple, so there are
        // 3 * 3 * 3 = 27 total options and this generates all of them.
        let all_combos = std::iter::repeat(vec![&a, &b, &c].into_iter())
            .take(3)
            .multi_cartesian_product();
        
        for points in all_combos {
            let p0 = points[0];
            let p1 = points[1];
            let p2 = points[2];
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