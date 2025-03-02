use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::{
    bounding_box::BoundingBox,
    error::FileError,
    line_segment::LineSegment, 
    point::Point, 
    triangle::Triangle,
    vertex::{Vertex, VertexId},
};


#[derive(Clone, Debug, PartialEq)]
pub struct Polygon {
    vertex_map: HashMap<VertexId, Vertex>,
}


impl Polygon {
    pub fn new(points: Vec<Point>) -> Polygon {
        let mut vertex_map = HashMap::new();

        // TODO currently the IDs are simply generated starting
        // at 0 and incrementing. If you want to keep this route,
        // will need to track index on self so that new vertices
        // could be added. Tried using unique_id::SequenceGenerator
        // but it was global which was harder to test with
        let num_points = points.len();
        let vertex_ids = (0..num_points)
            .map(VertexId::from)
            .collect::<Vec<_>>();

        for (i, point) in points.into_iter().enumerate() {
            let prev_id = vertex_ids[(i + num_points - 1) % num_points];
            let curr_id = vertex_ids[i];
            let next_id = vertex_ids[(i + num_points + 1) % num_points];
            let v = Vertex::new(point, curr_id, prev_id, next_id);
            vertex_map.insert(curr_id, v);
        }

        let polygon = Polygon { vertex_map };
        polygon.validate();
        polygon
    }

    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<Polygon, FileError> {
        let points_str: String = fs::read_to_string(path)?;
        let points: Vec<Point> = serde_json::from_str(&points_str)?;
        Ok(Polygon::new(points))
    }

    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), FileError>{
        let points = self.sorted_points();
        let points_str = serde_json::to_string_pretty(&points)?;
        fs::write(path, points_str)?;
        Ok(())
    }

    pub fn num_edges(&self) -> usize {
        self.edges().len()
    }

    pub fn num_vertices(&self) -> usize {
        self.vertex_map.len()
    }

    pub fn sorted_ids(&self) -> Vec<&VertexId> {
        let mut ids: Vec<_> = self.vertex_map.keys().collect();
        ids.sort();
        ids
    }

    pub fn sorted_points(&self) -> Vec<Point> {
        self.sorted_vertices()
            .iter()
            .map(|v| v.coords.clone())
            .collect::<Vec<Point>>()
    }

    pub fn vertices(&self) -> Vec<&Vertex> {
        self.vertex_map.values().collect_vec()
    }

    pub fn vertex_ids(&self) -> Vec<VertexId> {
        self.vertex_map.keys().cloned().collect_vec()
    }

    pub fn vertex_ids_set(&self) -> HashSet<VertexId> {
        self.vertex_map.keys().cloned().collect()
    }

    pub fn sorted_vertices(&self) -> Vec<&Vertex> {
        let mut vertices = self.vertices();
        vertices.sort_by(|a, b| a.id.cmp(&b.id));
        vertices
    }

    pub fn area(&self) -> f64 {
        let mut area = 0.0;
        let anchor = self.vertices()[0];
        for v1 in self.vertex_map.values() {
            let v2 = self.get_vertex(&v1.next).unwrap(); 
            area += Triangle::from_vertices(anchor, v1, v2).area();
        }
        area
    }

    pub fn remove_vertex(&mut self, id: &VertexId) -> Option<Vertex> {
        if let Some(v) = self.vertex_map.remove(id) {
            // TODO this would be an error condition if there was
            // a vertex for which prev/next weren't in the map,
            // instead of unwrap could have this func return 
            // result with an error tailored to this case
            self.get_vertex_mut(&v.prev).unwrap().next = v.next;
            self.get_vertex_mut(&v.next).unwrap().prev = v.prev;
            return Some(v);
        }
        None
    }

    pub fn get_vertex(&self, id: &VertexId) -> Option<&Vertex> {
        self.vertex_map.get(id)
    }

    pub fn get_vertex_mut(&mut self, id: &VertexId) -> Option<&mut Vertex> {
        self.vertex_map.get_mut(id)
    }

    pub fn get_point(&self, id: &VertexId) -> Option<Point> {
        if let Some(v) = self.get_vertex(id) {
            return Some(v.coords.clone())
        }
        None
    }

    pub fn get_line_segment(&self, id_1: &VertexId, id_2: &VertexId) -> Option<LineSegment> {
        if let Some(v1) = self.get_vertex(id_1) {
            if let Some(v2) = self.get_vertex(id_2) {
                return Some(LineSegment::from_vertices(v1, v2))
            }
        }
        None
    }

    pub fn get_triangle(&self, id_1: &VertexId, id_2: &VertexId, id_3: &VertexId) -> Option<Triangle> {
        if let Some(v1) = self.vertex_map.get(id_1) {
            if let Some(v2) = self.vertex_map.get(id_2) {
                if let Some(v3) = self.vertex_map.get(id_3) {
                    return Some(Triangle::from_vertices(v1, v2, v3));
                }
            }
        }
        None
    }

    pub fn edges(&self) -> HashSet<(VertexId, VertexId)> {
        // TODO could cache this and clear on modification
        let mut edges = HashSet::new();
        let anchor_id = self.vertices()[0].id;
        // TODO instead of unwrapping these, this function could
        // return result with an associated error type
        let mut current = self.get_vertex(&anchor_id).unwrap();
        loop {
            edges.insert((current.id, current.next));
            current = self.get_vertex(&current.next).unwrap();
            if current.id == anchor_id {
                break;
            }
        }
        edges
    }
    
    fn in_cone(&self, a: &Vertex, b: &Vertex) -> bool {
        let ab = LineSegment::from_vertices(a, b);
        let ba = &ab.reverse();
        // TODO instead of unwrap, return result with error
        let a0 = self.get_vertex(&a.prev).unwrap();
        let a1 = self.get_vertex(&a.next).unwrap();

        if a0.left_on(&LineSegment::from_vertices(a, a1)) {
            return a0.left(&ab) && a1.left(ba);
        }
        
        // Otherwise a is reflexive
        !(a1.left_on(&ab) && a0.left_on(ba))
    }
    
    pub fn diagonal(&self, a: &Vertex, b: &Vertex) -> bool {
        self.in_cone(a, b) && self.in_cone(b, a) && self.diagonal_internal_external(a, b)
    }

    fn diagonal_internal_external(&self, a: &Vertex, b: &Vertex) -> bool {
        let ab = &LineSegment::from_vertices(a, b);
        for (id1, id2) in self.edges() {
            // TODO instead of unwrap, return result with error
            let e = self.get_line_segment(&id1, &id2).unwrap();
            if !e.connected_to(ab) && e.intersects(ab) {
                return false;
            }
        } 
        true
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.min_x(), self.max_x(), self.min_y(), self.max_y())
    }

    pub fn min_x(&self) -> f64 {
        self.vertex_map.values().fold(f64::MAX, |acc, v| acc.min(v.coords.x))
    }

    pub fn max_x(&self) -> f64 {
        self.vertex_map.values().fold(f64::MIN, |acc, v| acc.max(v.coords.x))
    }

    pub fn min_y(&self) -> f64 {
        self.vertex_map.values().fold(f64::MAX, |acc, v| acc.min(v.coords.y))
    }

    pub fn max_y(&self) -> f64 {
        self.vertex_map.values().fold(f64::MIN, |acc, v| acc.max(v.coords.y))
    }

    pub fn leftmost_lowest_vertex(&self) -> &Vertex {
        let mut vertices = self.vertices();
        vertices.sort_by_key(|v| (OrderedFloat(v.coords.y), OrderedFloat(v.coords.x)));
        vertices[0]
    }

    pub fn rightmost_lowest_vertex(&self) -> &Vertex {
        let mut vertices = self.vertices();
        vertices.sort_by_key(|v| (OrderedFloat(v.coords.y), OrderedFloat(-v.coords.x)));
        vertices[0]
    }

    pub fn lowest_rightmost_vertex(&self) -> &Vertex {
        let mut vertices = self.vertices();
        vertices.sort_by_key(|v| (OrderedFloat(-v.coords.x), OrderedFloat(v.coords.y)));
        vertices[0]
    }

    pub fn highest_leftmost_vertex(&self) -> &Vertex {
        let mut vertices = self.vertices();
        vertices.sort_by_key(|v| (OrderedFloat(v.coords.x), OrderedFloat(-v.coords.y)));
        vertices[0]
    }

    pub fn translate(&mut self, x: f64, y: f64) {
        for v in self.vertex_map.values_mut() {
            v.translate(x, y);
        }
    }

    pub fn rotate_about_origin(&mut self, radians: f64) {
        for v in self.vertex_map.values_mut() {
            v.rotate_about_origin(radians);
        }    
    }

    pub fn rotate_about_point(&mut self, radians: f64, point: &Point) {
        for v in self.vertex_map.values_mut() {
            v.rotate_about_point(radians, point);
        }
    }

    pub fn round_coordinates(&mut self) {
        for v in self.vertex_map.values_mut() {
            v.round_coordinates();
        }
    }

    pub fn validate(&self) {
        self.validate_num_vertices();
        self.validate_cycle();
        self.validate_edge_intersections();
    }

    fn validate_num_vertices(&self) {
        let num_vertices = self.num_vertices();
        assert!(
            num_vertices >= 3,
            "Polygon must have at least 3 vertices, \
            this one has {num_vertices}"
        );
    }

    fn validate_cycle(&self) {
        // Walk the chain and terminate once a loop closure is
        // encountered, then validate every vertex was visited
        // once. Note the loop must terminate since there are
        // finite vertices and visited vertices are tracked.
        let anchor = self.vertices()[0];
        let mut current = anchor;
        let mut visited = HashSet::<VertexId>::new();

        loop {
            visited.insert(current.id);
            // TODO don't unwrap
            current = self.vertex_map.get(&current.next).unwrap();
            if current.id == anchor.id || visited.contains(&current.id) {
                break;
            }
        }

        let mut not_visited = HashSet::<VertexId>::new();
        for v in self.sorted_vertices() {
            if !visited.contains(&v.id) {
                not_visited.insert(v.id);
            }
        }
        assert!(
            not_visited.is_empty(),
            "Expected vertex chain to form a cycle but these \
            vertices were not visited: {not_visited:?}"
        );
    }

    fn validate_edge_intersections(&self) {
        let mut edges = Vec::new();
        let anchor_id = self.vertices()[0].id;
        let mut current = self.get_vertex(&anchor_id).unwrap();
        loop {
            let next = self.get_vertex(&current.next).unwrap();
            let ls = LineSegment::from_vertices(current, next);
            edges.push(ls);
            current = next;
            if current.id == anchor_id {
                break;
            }
        }
        
        for i in 0..(edges.len() - 1) {
            let e1 = &edges[i];
            // Adjacent edges should share a common vertex
            assert!(e1.incident_to(edges[i+1].p1));
            for e2 in edges.iter().take(edges.len() -1).skip(i+2) {
                // Non-adjacent edges should have no intersection
                assert!(!e1.intersects(e2));
                assert!(!e1.incident_to(e2.p1));
                assert!(!e1.incident_to(e2.p2));
                assert!(!e2.intersects(e1));
                assert!(!e2.incident_to(e1.p1));
                assert!(!e2.incident_to(e1.p2));
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::F64_ASSERT_PRECISION;
    use crate::test_util::*;

    use super::*;
    use assert_approx_eq::assert_approx_eq;
    use rstest::rstest;
    use rstest_reuse::{self, *};
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8, PI, SQRT_2};
    use tempfile::NamedTempFile;


    #[test]
    #[should_panic]
    fn test_invalid_polygon_not_enough_vertices() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);
        let points = vec![p1, p2];
        let polygon = Polygon::new(points);
        assert_eq!(2, polygon.num_vertices());
    }

    #[test]
    #[should_panic]
    fn test_invalid_polygon_not_simple() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(2.0, 0.0);
        let p3 = Point::new(2.0, 2.0);
        let p4 = Point::new(0.0, 2.0);
        let p5 = Point::new(4.0, 1.0); // This one should break it
        let points = vec![p1, p2, p3, p4, p5];
        let polygon = Polygon::new(points);
        assert_eq!(3, polygon.num_vertices())
    }

    #[apply(all_polygons)]
    fn test_json(case: PolygonTestCase) {
        let filename = NamedTempFile::new()
            .unwrap()
            .into_temp_path();
        let _ = case.polygon.to_json(&filename);
        let new_polygon = Polygon::from_json(&filename).unwrap();
        assert_eq!(case.polygon, new_polygon);
    }

    #[test]
    // TODO could expand this test to polygon cases
    fn test_min_max() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(5.0, -1.0);
        let p3 = Point::new(7.0, 6.0);
        let p4 = Point::new(-4.0, 8.0);
        let p5 = Point::new(-2.0, -3.0);
        let points = vec![p1, p2, p3, p4, p5];
        let polygon = Polygon::new(points);
        assert_eq!(polygon.min_x(), -4.0);
        assert_eq!(polygon.max_x(), 7.0);
        assert_eq!(polygon.min_y(), -3.0);
        assert_eq!(polygon.max_y(), 8.0);
    }

    #[test]
    // TODO could expand this test to polygon cases
    fn test_lowest_vertex() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(5.0, -1.0);
        let p3 = Point::new(7.0, 6.0);
        let p4 = Point::new(-4.0, 8.0);
        let p5 = Point::new(-2.0, -3.0);
        let points = vec![p1, p2, p3, p4, p5];
        let polygon = Polygon::new(points);
        let lowest = polygon.leftmost_lowest_vertex();
        assert_eq!(lowest.coords.x, -2.0);
        assert_eq!(lowest.coords.y, -3.0);
    }

    #[apply(all_polygons)]
    fn test_area(case: PolygonTestCase) {
        let area = case.polygon.area();
        assert_eq!(area, case.metadata.area);
    }

    #[apply(all_polygons)]
    fn test_edges(case: PolygonTestCase) {
        let mut expected_edges = HashSet::new();
        for i in 0usize..case.metadata.num_edges {
            let src = VertexId::from(i);
            let dest = VertexId::from((i + 1) % case.metadata.num_edges);
            expected_edges.insert((src, dest));
        }
        let edges = case.polygon.edges();
        assert_eq!(edges.len(), case.metadata.num_edges);
        assert_eq!(edges, expected_edges);
    }

    #[apply(all_polygons)]
    fn test_rotation_about_origin(
        case: PolygonTestCase, 
        #[values(PI, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8)] radians: f64
    ) {
        let mut polygon = case.polygon;
        polygon.rotate_about_origin(radians);
        polygon.validate();
        assert_eq!(polygon.num_edges(), case.metadata.num_edges);
        assert_eq!(polygon.num_vertices(), case.metadata.num_vertices);
        assert_approx_eq!(polygon.area(), case.metadata.area, F64_ASSERT_PRECISION);
    }

    #[apply(all_polygons)]
    fn test_rotation_about_point(
        case: PolygonTestCase,
        #[values(PI, FRAC_PI_2, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, FRAC_PI_8)] radians: f64,
        #[values(Point::new(5.2, 10.0), Point::new(-43.0, PI), Point::new(SQRT_2, 1e8))] point: Point
    ) {
        let mut polygon = case.polygon;
        polygon.rotate_about_point(radians, &point);
        assert_eq!(polygon.num_edges(), case.metadata.num_edges);
        assert_eq!(polygon.num_vertices(), case.metadata.num_vertices);
        assert_approx_eq!(polygon.area(), case.metadata.area, F64_ASSERT_PRECISION);
    }

    #[apply(all_polygons)]
    fn test_attributes(case: PolygonTestCase) {
        assert_eq!(case.polygon.num_edges(), case.metadata.num_edges);
        assert_eq!(case.polygon.num_vertices(), case.metadata.num_vertices);

        // Not all test cases have these defined so only assert on ones that do
        let num_extreme_points = case.metadata.extreme_points.len();
        let num_interior_points = case.metadata.interior_points.len();
        if num_extreme_points > 0 || num_interior_points > 0 {
            assert_eq!(
                num_extreme_points + num_interior_points, 
                case.metadata.num_vertices,
            );
            assert!(
                case.metadata.extreme_points.is_disjoint(&case.metadata.interior_points)
            );
        }
        // This meta-assert is only valid for polygons without holes, holes 
        // are not yet supported. Will need a flag in the metadata to know 
        // if holes are present and then this assert would be conditional
        assert_eq!(case.metadata.num_edges, case.metadata.num_vertices);
    }
}