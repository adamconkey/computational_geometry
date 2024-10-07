use itertools::Itertools;
use std::collections::HashMap;

use crate::{
    line_segment::LineSegment,
    triangle::Triangle,
    vertex::Vertex,
    vertex_map::VertexMap,
};


pub struct Polygon<'a> {
    anchor: &'a Vertex,
    neighbors: HashMap<&'a Vertex, (&'a Vertex, &'a Vertex)>,
}


impl<'a> Polygon<'a> {
    pub fn new(vertices: Vec<&'a Vertex>) -> Polygon<'a> {
        let mut neighbors = HashMap::new();

        let first = &vertices[0];
        let last = vertices
            .last()
            .expect("Polygon should have at least 3 vertices");

        // TODO I suspect I'm doing something silly here, having to deref
        // everything. I think it has to do with me storing a vec of refs
        // and then doing iter as opposed to into_iter()
        
        for (v0, v1, v2) in vertices.iter().tuple_windows::<(_,_,_)>() {
            neighbors.insert(*v1, (*v0, *v2));

            if v0 == first {
                neighbors.insert(*v0, (*last, *v1));
            }
            if v2 == last {
                neighbors.insert(*v2, (*v1, *first));
            }
        }

        Polygon { anchor: &vertices[0], neighbors }
    }

    pub fn from_vmap(vmap: &'a VertexMap) -> Polygon<'a> {
        Polygon::new(vmap.all_vertices())
    }
    
    pub fn double_area(&self) -> i32 {
        // The first edge will include the anchor, but that area
        // ends up being zero since v2 will trivially be collinear
        // with anchor-v1 and thus doesn't affect the compuation
        let mut area = 0;
        for e in self.edges() {
            area += Triangle::new(self.anchor, e.v1, e.v2).double_area();
        }
        area
    }

    pub fn triangulation(&self) -> Vec<LineSegment> {
        let mut triangulation = Vec::new();
        let mut neighbors = self.neighbors.clone();

        while neighbors.len() > 3 {
            let mut v2 = self.anchor;
            
            loop {
                let (v1, v3) = neighbors
                    .get(v2)
                    .expect("Every vertex should have neighbors stored");

                if self.diagonal(&LineSegment::new(v1, v3)) {
                    // We found an ear, need to add to the triangulation,
                    // remove the vertex, and update the neighbor map

                    let (_prev, v4) = neighbors
                        .get(v3)
                        .expect("Every vertex should have neighbors stored");
                    let (v0, _next) = neighbors
                        .get(v1)
                        .expect("Every vertex should have neighbors stored");

                    triangulation.push(LineSegment::new(v1, v2));

                    // TODO I have some map issues all over here, not sure 
                    // yet how to resolve it. Perhaps my map tricks are 
                    // breaking down and need to rethink the data structure.
                    // Also this is not ergonomic at all when you just need
                    // to update the refs for just the prev or just the next 
                    // so I think this data structure is misguided a bit. 
                    // Will need to think more on what the best way to 
                    // proceed here is.
                    neighbors.insert(v1, (v0, v3));

                    neighbors.insert(v3, (v1, v4));
                }

                // TODO this obviously seems a bit silly, would otherwise just want to 
                // set v2 = v3 since we know v3 to be v2's next neighbor, but because
                // of the possible insertion of v3 in the map above we have some
                // borrowing issues. Not sure yet if there's a smarter way to go.
                let (_prev, v2) = neighbors
                    .get(v2)
                    .expect("Every vertex should have neighbors stored");

                if v2 == &self.anchor {
                    // Made a full pass through all vertices, can advance to
                    // next iter of outer loop
                    break;
                }
            }
        }

        triangulation
    }

    pub fn edges(&self) -> Vec<LineSegment> {
        // TODO would be cool to cache this
        let mut edges = Vec::new();
        let mut current = self.anchor;

        // Do forward pass through hashmap to get all ordered edges
        loop {
            let (_prev, next) = self.neighbors
                .get(current)
                .expect("Every vertex should have neighbors stored");
            edges.push(LineSegment::new(current, next));

            current = next;
            
            if current == self.anchor {
                break;
            }
        }

        edges
    }

    pub fn neighbors(&self, v: &Vertex) -> (&Vertex, &Vertex) {
        *self.neighbors
            .get(v)
            .expect("Every vertex should have neighbors stored")
        
    }
    
    pub fn in_cone(&self, ab: &LineSegment) -> bool {
        let a = ab.v1;
        let ba = &ab.reverse();
        let (a0, a1) = self.neighbors(a);

        if a0.left_on(&LineSegment::new(a, a1)) {
            return a0.left(ab) && a1.left(ba);
        }
        
        // Otherwise a is reflexive
        !(a1.left_on(ab) && a0.left_on(ba))
    }
    
    pub fn diagonal(&self, ab: &LineSegment) -> bool {
        let ba = &ab.reverse();
        self.in_cone(ab) && self.in_cone(ba) && self.diagonal_internal_external(ab)
    }

    fn diagonal_internal_external(&self, ab: &LineSegment) -> bool {
        for e in self.edges() {
            if !e.connected_to(ab) && e.intersects(ab) {
                return false;
            }
        } 
        true
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use std::str::FromStr;

    // TODO I think it will be better to ultimately read these
    // from file since I'll likely have some with many vertices
    // which will get a little unwieldy here.
    #[fixture]
    fn polygon_1() -> &'static str {
        concat!("0 0 a\n", "3 4 b\n", "6 2 c\n", "7 6 d\n", "3 9 e\n", "-2 7 f")
    }

    #[fixture]
    fn polygon_2() -> &'static str {
        concat!(
            "0 0 0\n",
            "12 9 1\n",
            "14 4 2\n",
            "24 10 3\n",
            "14 24 4\n",
            "11 17 5\n",
            "13 19 6\n",
            "16 12 7\n",
            "9 13 8\n",
            "7 18 9\n",
            "11 21 10\n",
            "8 24 11\n",
            "1 22 12\n",
            "2 18 13\n",
            "4 20 14\n",
            "6 10 15\n",
            "-2 11 16\n",
            "6 6 17"
        )
    }

    #[fixture]
    fn right_triangle() -> &'static str {
        concat!("0 0 a\n", "3 0 b\n", "0 4 c")
    }

    #[fixture]
    fn square_4x4() -> &'static str {
        concat!("0 0 a\n", "4 0 b\n", "4 4 c\n", "0 4 d")
    }
    
    #[rstest]
    // TODO now that this is parametrized, can add as many polygons
    // here as possible to get meaningful tests on area
    #[case(right_triangle(), 12)]
    #[case(polygon_2(), 454)]
    fn test_area(#[case] polygon_str: &str, #[case] expected_double_area: i32) {
        let vmap = VertexMap::from_str(polygon_str).unwrap();        
        let polygon = Polygon::from_vmap(&vmap);
        let double_area = polygon.double_area();
        assert_eq!(double_area, expected_double_area);
    }

    #[rstest]
    fn test_neighbors_square(square_4x4: &str) {
        let vmap = VertexMap::from_str(square_4x4).unwrap();
        let polygon = Polygon::from_vmap(&vmap);

        let a = vmap.get("a").unwrap();
        let b = vmap.get("b").unwrap();
        let c = vmap.get("c").unwrap();
        let d = vmap.get("d").unwrap();
        
        assert_eq!(polygon.neighbors(a), (d, b));
        assert_eq!(polygon.neighbors(b), (a, c));
        assert_eq!(polygon.neighbors(c), (b, d));
        assert_eq!(polygon.neighbors(d), (c, a));
    }

    #[rstest]
    fn test_neighbors_p1(polygon_1: &str) {
        let vmap = VertexMap::from_str(polygon_1).unwrap();
        let polygon = Polygon::from_vmap(&vmap);

        let a = vmap.get("a").unwrap();
        let b = vmap.get("b").unwrap();
        let c = vmap.get("c").unwrap();
        let d = vmap.get("d").unwrap();
        let e = vmap.get("e").unwrap();
        let f = vmap.get("f").unwrap();
        
        assert_eq!(polygon.neighbors(a), (f, b));
        assert_eq!(polygon.neighbors(b), (a, c));
        assert_eq!(polygon.neighbors(c), (b, d));
        assert_eq!(polygon.neighbors(d), (c, e));
        assert_eq!(polygon.neighbors(e), (d, f));
        assert_eq!(polygon.neighbors(f), (e, a));
    }
    
    #[rstest]
    fn test_diagonal(polygon_1: &str) {
        let vmap = VertexMap::from_str(polygon_1).unwrap();
        let polygon = Polygon::from_vmap(&vmap);

        let a = vmap.get("a").unwrap();
        let b = vmap.get("b").unwrap();
        let c = vmap.get("c").unwrap();
        let d = vmap.get("d").unwrap();
        let e = vmap.get("e").unwrap();
        let f = vmap.get("f").unwrap();
    
        let ac = LineSegment::new(a, c);
        let ad = LineSegment::new(a, d);
        let ae = LineSegment::new(a, e);
        let bd = LineSegment::new(b, d);
        let be = LineSegment::new(b, e);
        let bf = LineSegment::new(b, f);
        let ca = LineSegment::new(c, a);
        let ce = LineSegment::new(c, e);
        let cf = LineSegment::new(c, f);
        let da = LineSegment::new(d, a);
        let db = LineSegment::new(d, b);
        let df = LineSegment::new(d, f);
        let ea = LineSegment::new(e, a);
        let eb = LineSegment::new(e, b);
        let ec = LineSegment::new(e, c);
        let fb = LineSegment::new(f, b);
        let fc = LineSegment::new(f, c);
        let fd = LineSegment::new(f, d);

        let internal = vec![&ae, &bd, &be, &bf, &ce, &db, &df, &ea, &eb, &ec, &fb, &fd];
        let external = vec![&ac, &ca];
        let not_diagonal = vec![&ad, &cf, &da, &fc];
        
        for ls in internal {
            assert!(polygon.in_cone(ls));
            assert!(polygon.diagonal_internal_external(ls));
            assert!(polygon.diagonal(ls));
        }

        for ls in external {
            // TODO might want to think of another example and think carefully
            // about the in_cone, I think there'd be examples where at least
            // one of the directions fails
            assert!(!polygon.in_cone(ls));
            assert!( polygon.diagonal_internal_external(ls));
            assert!(!polygon.diagonal(ls));
        }

        for ls in not_diagonal {
            assert!(!polygon.diagonal_internal_external(ls));
            assert!(!polygon.diagonal(ls));
        }
    }
}
