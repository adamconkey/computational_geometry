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

    pub fn vertices(&self) -> Vec<&Vertex> {
        // TODO can rethink edges method if this works

        let mut vertices = Vec::new();
        let mut current = self.anchor;

        loop {
            vertices.push(current);
            current = self.neighbors
                .get(current)
                .expect("Every vertex should have neighbors stored")
                .1;
            
            if current == self.anchor {
                break;
            }
        }

        vertices
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

        while neighbors.len() > 3 {
            let mut v2 = anchor;
            
            loop {
                // Removing instead of borrowing, so that we don't run into 
                // immutable borrow problems. It will be inserted again if 
                // we end up not finding an ear. May want to rethink this 
                // if there's a better way to go about so we're not 
                // unnecessarily removing things.
                let (v1, v3) = neighbors
                    .remove(v2)
                    .expect("Every vertex should have neighbors stored");

                if self.diagonal(&LineSegment::new(v1, v3)) {
                    // We found an ear, add to the triangulation
                    triangulation.push(LineSegment::new(v1, v3));

                    let v4 = neighbors
                        .get(v3)
                        .expect("Every vertex should have neighbors stored")
                        .1;
                    let v0 = neighbors
                        .get(v1)
                        .expect("Every vertex should have neighbors stored")
                        .0;

                    // The ear vertex has been removed, update its neighbors 
                    // so that their neighbors point to the correct vertices
                    neighbors.insert(v1, (v0, v3));
                    neighbors.insert(v3, (v1, v4));
                    anchor = v3;  // In case removed was anchor
                }
                else {
                    // This wasn't an ear, so re-insert into neighbor map
                    neighbors.insert(v2, (v1, v3));
                }

                v2 = v3;  // Advance to next vertex in chain

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
    
    // TODO save these in files and load here
    #[fixture]
    fn polygon_1() -> VertexMap {
        let polygon_str = r#"{
            "vertices": {
                "a": {"x":  0, "y": 0, "id": "a"},
                "b": {"x":  3, "y": 4, "id": "b"},
                "c": {"x":  6, "y": 2, "id": "c"},
                "d": {"x":  7, "y": 6, "id": "d"},
                "e": {"x":  3, "y": 9, "id": "e"},
                "f": {"x": -2, "y": 7, "id": "f"}
            }
        }"#;
        serde_json::from_str(&polygon_str).unwrap()
    }

    #[fixture]
    fn polygon_2() -> VertexMap {
        let polygon_str = r#"{
            "vertices": {
                "0":  {"x":  0, "y":  0, "id": "0"},
                "1":  {"x": 12, "y":  9, "id": "1"},
                "2":  {"x": 14, "y":  4, "id": "2"},
                "3":  {"x": 24, "y": 10, "id": "3"},
                "4":  {"x": 14, "y": 24, "id": "4"},
                "5":  {"x": 11, "y": 17, "id": "5"},
                "6":  {"x": 13, "y": 19, "id": "6"},
                "7":  {"x": 14, "y": 12, "id": "7"},
                "8":  {"x":  9, "y": 13, "id": "8"},
                "9":  {"x":  7, "y": 18, "id": "9"},
                "10": {"x": 11, "y": 21, "id": "10"},
                "11": {"x":  8, "y": 24, "id": "11"},
                "12": {"x":  1, "y": 22, "id": "12"},
                "13": {"x":  2, "y": 18, "id": "13"},
                "14": {"x":  4, "y": 20, "id": "14"},
                "15": {"x":  6, "y": 10, "id": "15"},
                "16": {"x": -2, "y": 11, "id": "16"},
                "17": {"x":  6, "y":  6, "id": "17"}
            }
        }"#;
        serde_json::from_str(&polygon_str).unwrap()
    }

    #[fixture]
    fn right_triangle() -> VertexMap {
        let polygon_str = r#"{
            "vertices": {
                "a": {"x": 0, "y": 0, "id": "a"},
                "b": {"x": 3, "y": 0, "id": "b"},
                "c": {"x": 0, "y": 4, "id": "c"}
            }
        }"#;
        serde_json::from_str(&polygon_str).unwrap()
    }

    #[fixture]
    fn square_4x4() -> VertexMap {
        let polygon_str = r#"{
            "vertices": {
                "a": {"x":  0, "y": 0, "id": "a"},
                "b": {"x":  4, "y": 0, "id": "b"},
                "c": {"x":  4, "y": 4, "id": "c"},
                "d": {"x":  0, "y": 4, "id": "d"}
            }
        }"#;
        serde_json::from_str(&polygon_str).unwrap()
    }


    #[rstest]
    // TODO now that this is parametrized, can add as many polygons
    // here as possible to get meaningful tests on area
    #[case(right_triangle(), 12)]
    #[case(polygon_2(), 466)]
    fn test_area(#[case] vmap: VertexMap, #[case] expected_double_area: i32) {
        let polygon = Polygon::from_vmap(&vmap);
        let double_area = polygon.double_area();
        assert_eq!(double_area, expected_double_area);
    }

    #[rstest]
    fn test_neighbors_square(square_4x4: VertexMap) {
        let polygon = Polygon::from_vmap(&square_4x4);

        let a = square_4x4.get("a").unwrap();
        let b = square_4x4.get("b").unwrap();
        let c = square_4x4.get("c").unwrap();
        let d = square_4x4.get("d").unwrap();
        
        assert_eq!(polygon.neighbors(a), (d, b));
        assert_eq!(polygon.neighbors(b), (a, c));
        assert_eq!(polygon.neighbors(c), (b, d));
        assert_eq!(polygon.neighbors(d), (c, a));
    }

    #[rstest]
    fn test_edges_square(square_4x4: VertexMap) {
        let polygon = Polygon::from_vmap(&square_4x4);

        let expected_edges = vec![
            square_4x4.get_line_segment("a", "b"),
            square_4x4.get_line_segment("b", "c"),
            square_4x4.get_line_segment("c", "d"),
            square_4x4.get_line_segment("d", "a"),
        ];
    
        assert_eq!(polygon.edges(), expected_edges);
    }

    #[rstest]
    fn test_vertices_square(square_4x4: VertexMap) {
        let polygon = Polygon::from_vmap(&square_4x4);

        let expected_vertices = vec![
            square_4x4.get("a").unwrap(),
            square_4x4.get("b").unwrap(),
            square_4x4.get("c").unwrap(),
            square_4x4.get("d").unwrap(),
        ];
    
        assert_eq!(polygon.vertices(), expected_vertices);
    }

    #[rstest]
    fn test_neighbors_p1(polygon_1: VertexMap) {
        let polygon = Polygon::from_vmap(&polygon_1);

        let a = polygon_1.get("a").unwrap();
        let b = polygon_1.get("b").unwrap();
        let c = polygon_1.get("c").unwrap();
        let d = polygon_1.get("d").unwrap();
        let e = polygon_1.get("e").unwrap();
        let f = polygon_1.get("f").unwrap();
        
        assert_eq!(polygon.neighbors(a), (f, b));
        assert_eq!(polygon.neighbors(b), (a, c));
        assert_eq!(polygon.neighbors(c), (b, d));
        assert_eq!(polygon.neighbors(d), (c, e));
        assert_eq!(polygon.neighbors(e), (d, f));
        assert_eq!(polygon.neighbors(f), (e, a));
    }
    
    #[rstest]
    fn test_edges_p1(polygon_1: VertexMap) {
        let polygon = Polygon::from_vmap(&polygon_1);

        let expected_edges = vec![
            polygon_1.get_line_segment("a", "b"),
            polygon_1.get_line_segment("b", "c"),
            polygon_1.get_line_segment("c", "d"),
            polygon_1.get_line_segment("d", "e"),
            polygon_1.get_line_segment("e", "f"),
            polygon_1.get_line_segment("f", "a"),
        ];
        
        assert_eq!(polygon.edges(), expected_edges);
    }

    #[rstest]
    fn test_vertices_p1(polygon_1: VertexMap) {
        let polygon = Polygon::from_vmap(&polygon_1);

        let expected_vertices = vec![
            polygon_1.get("a").unwrap(),
            polygon_1.get("b").unwrap(),
            polygon_1.get("c").unwrap(),
            polygon_1.get("d").unwrap(),
            polygon_1.get("e").unwrap(),
            polygon_1.get("f").unwrap(),
        ];
        
        assert_eq!(polygon.vertices(), expected_vertices);
    }

    #[rstest]
    fn test_diagonal(polygon_1: VertexMap) {
        let polygon = Polygon::from_vmap(&polygon_1);

        let ac = polygon_1.get_line_segment("a", "c");
        let ad = polygon_1.get_line_segment("a", "d");
        let ae = polygon_1.get_line_segment("a", "e");
        let bd = polygon_1.get_line_segment("b", "d");
        let be = polygon_1.get_line_segment("b", "e");
        let bf = polygon_1.get_line_segment("b", "f");
        let ca = polygon_1.get_line_segment("c", "a");
        let ce = polygon_1.get_line_segment("c", "e");
        let cf = polygon_1.get_line_segment("c", "f");
        let da = polygon_1.get_line_segment("d", "a");
        let db = polygon_1.get_line_segment("d", "b");
        let df = polygon_1.get_line_segment("d", "f");
        let ea = polygon_1.get_line_segment("e", "a");
        let eb = polygon_1.get_line_segment("e", "b");
        let ec = polygon_1.get_line_segment("e", "c");
        let fb = polygon_1.get_line_segment("f", "b");
        let fc = polygon_1.get_line_segment("f", "c");
        let fd = polygon_1.get_line_segment("f", "d");

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
    fn test_triangulation(polygon_2: VertexMap) {
        let polygon = Polygon::from_vmap(&polygon_2);
        let triangulation = polygon.triangulation();
        
        let ls_17_1 = polygon_2.get_line_segment("17", "1");
        let ls_1_3 = polygon_2.get_line_segment("1", "3");
        let ls_4_6 = polygon_2.get_line_segment("4", "6");
        let ls_4_7 = polygon_2.get_line_segment("4", "7");
        let ls_9_11 = polygon_2.get_line_segment("9", "11");
        let ls_12_14 = polygon_2.get_line_segment("12", "14");
        let ls_15_17 = polygon_2.get_line_segment("15", "17");
        let ls_15_1 = polygon_2.get_line_segment("15", "1");
        let ls_15_3 = polygon_2.get_line_segment("15", "3");
        let ls_3_7 = polygon_2.get_line_segment("3", "7");
        let ls_11_14 = polygon_2.get_line_segment("11", "14");
        let ls_15_7 = polygon_2.get_line_segment("15", "7");
        let ls_15_8 = polygon_2.get_line_segment("15", "8");
        let ls_15_9 = polygon_2.get_line_segment("15", "9");
        let ls_9_14 = polygon_2.get_line_segment("9", "14");
       
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
