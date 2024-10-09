use itertools::Itertools;
use std::collections::HashMap;

use crate::{
    line_segment::LineSegment,
    triangle::Triangle,
    vertex::Vertex,
    vertex_map::VertexMap,
};


#[derive(Clone)]
pub struct NeighborMap<'a> {
    neighbors: HashMap<&'a Vertex, (&'a Vertex, &'a Vertex)>,
}

impl<'a> NeighborMap<'a> {
    pub fn new(vertices: Vec<&'a Vertex>) -> NeighborMap<'a> {
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

        NeighborMap { neighbors }
    }

    pub fn len(&self) -> usize {
        self.neighbors.len()
    }

    pub fn get(&self, v: &Vertex) -> (&Vertex, &Vertex) {
        *self.neighbors
            .get(v)
            .expect("Every vertex should have its neighbor stored")
    }

    pub fn insert(&mut self, v: &'a Vertex, neighbors: (&'a Vertex, &'a Vertex)) {
        self.neighbors.insert(v, neighbors);
    }

    pub fn remove(&mut self, v: &Vertex) {
        self.neighbors.remove(v);
    }
}


pub struct Polygon<'a> {
    anchor: &'a Vertex,
    neighbors: NeighborMap<'a>,
}


impl<'a> Polygon<'a> {
    pub fn new(vertices: Vec<&'a Vertex>) -> Polygon<'a> {
        Polygon { anchor: &vertices[0], neighbors: NeighborMap::new(vertices) }
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
        let mut anchor = self.anchor;

        // TODO tried to port to using a neighbormap so that it can offer
        // more compact retrieval of prev/next but am having several 
        // borrow errors here. Need to figure that out. Then should be 
        // able to make convenience funcs to retrieve prev and next so
        // that you don't always have to retrieve the full tuple and 
        // index to it.
        fn name(b: &mut test::Bencher) {
            b.iter(|| /* benchmark code */)
        }
        while neighbors.len() > 3 {
            let mut v2 = anchor;
            
            loop {
                let (v1, v3) = neighbors.get(v2);

                if self.diagonal(&LineSegment::new(v1, v3)) {
                    // We found an ear, need to add to the triangulation,
                    // remove the vertex, and update the neighbor map

                    let v4 = neighbors.get(v3).1;
                    let v0 = neighbors.get(v1).0;

                    triangulation.push(LineSegment::new(v1, v3));

                    neighbors.insert(v1, (v0, v3));
                    neighbors.insert(v3, (v1, v4));
                    neighbors.remove(v2);
                    anchor = v3;  // In case removed was anchor
                }

                v2 = v3;

                if v2 == anchor {
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
            // TODO use more compact retrieval
            let (_prev, next) = self.neighbors.get(current);
            edges.push(LineSegment::new(current, next));

            current = next;
            
            if current == self.anchor {
                break;
            }
        }

        edges
    }

    pub fn in_cone(&self, ab: &LineSegment) -> bool {
        let a = ab.v1;
        let ba = &ab.reverse();
        let (a0, a1) = self.neighbors.get(a);

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
            "14 12 7\n",
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
    #[case(polygon_2(), 466)]
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
        
        assert_eq!(polygon.neighbors.get(a), (d, b));
        assert_eq!(polygon.neighbors.get(b), (a, c));
        assert_eq!(polygon.neighbors.get(c), (b, d));
        assert_eq!(polygon.neighbors.get(d), (c, a));
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
        
        assert_eq!(polygon.neighbors.get(a), (f, b));
        assert_eq!(polygon.neighbors.get(b), (a, c));
        assert_eq!(polygon.neighbors.get(c), (b, d));
        assert_eq!(polygon.neighbors.get(d), (c, e));
        assert_eq!(polygon.neighbors.get(e), (d, f));
        assert_eq!(polygon.neighbors.get(f), (e, a));
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

    #[rstest]
    fn test_triangulation(polygon_2: &str) {
        let vmap = VertexMap::from_str(polygon_2).unwrap();        
        let polygon = Polygon::from_vmap(&vmap);
        let triangulation = polygon.triangulation();
        
        // TODO maybe add utility to just retrieve line segments from vmap 
        // so you don't have to get all of the vertices here
        let v1 = vmap.get("1").unwrap();
        let v3 = vmap.get("3").unwrap();
        let v4 = vmap.get("4").unwrap();
        let v6 = vmap.get("6").unwrap();
        let v7 = vmap.get("7").unwrap();
        let v8 = vmap.get("8").unwrap();
        let v9 = vmap.get("9").unwrap();
        let v11 = vmap.get("11").unwrap();
        let v12 = vmap.get("12").unwrap();
        let v14 = vmap.get("14").unwrap();
        let v15 = vmap.get("15").unwrap();
        let v17 = vmap.get("17").unwrap();

        let ls_17_1 = LineSegment::new(v17, v1);
        let ls_1_3 = LineSegment::new(v1, v3);
        let ls_4_6 = LineSegment::new(v4, v6);
        let ls_4_7 = LineSegment::new(v4, v7);
        let ls_9_11 = LineSegment::new(v9, v11);
        let ls_12_14 = LineSegment::new(v12, v14);
        let ls_15_17 = LineSegment::new(v15, v17);
        let ls_15_1 = LineSegment::new(v15, v1);
        let ls_15_3 = LineSegment::new(v15, v3);
        let ls_3_7 = LineSegment::new(v3, v7);
        let ls_11_14 = LineSegment::new(v11, v14);
        let ls_15_7 = LineSegment::new(v15, v7);
        let ls_15_8 = LineSegment::new(v15, v8);
        let ls_15_9 = LineSegment::new(v15, v9);
        let ls_9_14 = LineSegment::new(v9, v14);
       
        let expected = vec![
            ls_17_1,
            ls_1_3,
            ls_4_6,
            ls_4_7,
            ls_9_11,
            ls_12_14,
            ls_15_17,
            ls_15_1,
            ls_15_3,
            ls_3_7,
            ls_11_14,
            ls_15_7,
            ls_15_8,
            ls_15_9,
            ls_9_14
        ];

        assert_eq!(expected, triangulation);

    }
}
