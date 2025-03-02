use std::{fmt, slice::Iter};

use crate::{polygon::Polygon, vertex::VertexId};


#[derive(Debug, Clone)]
pub struct EarNotFoundError;

impl fmt::Display for EarNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "polygon is likely invalid")
    }
}


#[derive(Eq, Hash, PartialEq)]
pub struct TriangleVertexIds(pub VertexId, pub VertexId, pub VertexId);


#[derive(Default)]
pub struct Triangulation {
    triangles: Vec<TriangleVertexIds>,
}

impl Triangulation {
    pub fn push(&mut self, value: TriangleVertexIds) {
        self.triangles.push(value)
    }

    pub fn iter(&self) -> Iter<'_, TriangleVertexIds> { 
        self.triangles.iter()
    }

    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }
}


pub trait TriangulationComputer {
    fn triangulation(&self, polygon: &Polygon) -> Triangulation;
}


#[derive(Default)]
pub struct EarClipping;

impl EarClipping {
    fn find_ear(&self, polygon: &Polygon) -> Result<VertexId, EarNotFoundError> {
        for v in polygon.vertices() {
            let prev = polygon.get_vertex(&v.prev).unwrap();
            let next = polygon.get_vertex(&v.next).unwrap();
            if polygon.diagonal(prev, next) {
                return Ok(v.id);
            }
        }
        Err(EarNotFoundError)
    }
}

impl TriangulationComputer for EarClipping {
    fn triangulation(&self, polygon: &Polygon) -> Triangulation {
        let mut triangulation = Triangulation::default();
        let mut polygon = polygon.clone();

        while polygon.num_vertices() > 3 {
            let id = self.find_ear(&polygon)
                .expect("valid polygons with 3 or more vertices should have an ear");
            // TODO instead of unwrap, return result with error
            let v = polygon.remove_vertex(&id).unwrap();
            triangulation.push(TriangleVertexIds(v.prev, id, v.next));
        }
        // At this stage there should be exactly 3 vertices left,
        // which form the final triangle of the triangulation
        let v = polygon.vertices()[0];
        triangulation.push(TriangleVertexIds(v.prev, v.id, v.next));

        triangulation
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::{self, *};
    use crate::test_util::*;

    #[apply(all_polygons)]
    fn test_triangulation(
        #[case] 
        case: PolygonTestCase, 
        #[values(EarClipping)]
        computer: impl TriangulationComputer
    ) {
        let triangulation = computer.triangulation(&case.polygon);
        assert_eq!(triangulation.len(), case.metadata.num_triangles);
        // This meta-assert is only valid for polygons without holes, holes 
        // are not yet supported. Will need a flag in the metadata to know 
        // if holes are present and then this assert would be conditional
        assert_eq!(case.metadata.num_triangles, case.metadata.num_edges - 2);

        // Check that the aggregated area over the triangles is as expected
        let mut triangulation_area = 0.0;
        for ids in triangulation.iter() {
            let t = case.polygon.get_triangle(&ids.0, &ids.1, &ids.2).unwrap();
            triangulation_area += t.area();
        }
        assert_eq!(triangulation_area, case.metadata.area);
    }
}