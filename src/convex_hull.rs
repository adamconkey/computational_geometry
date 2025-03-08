use itertools::Itertools;
use ordered_float::OrderedFloat as OF;
use std::collections::HashSet;

use crate::{
    line_segment::LineSegment, 
    polygon::Polygon, 
    vertex::VertexId
};


#[derive(Debug)]
pub struct ConvexHull {
    vertices: HashSet<VertexId>,
}

impl ConvexHull {
    pub fn new(vertices: HashSet<VertexId>) -> Self {
        ConvexHull { vertices }
    }

    pub fn add_vertex(&mut self, vertex_id: VertexId) {
        self.vertices.insert(vertex_id);
    }

    pub fn add_vertices(&mut self, vertices: impl IntoIterator<Item = VertexId>) {
        for v in vertices {
            self.add_vertex(v);
        }
    }

    pub fn get_vertices(&self) -> Vec<VertexId> {
        // TODO this is a little silly to sort on every retrieval 
        // especially if nothing changes. I originally had a 
        // min-heap but found you still had to clone/sort to get 
        // the whole vec (not just peek/pop from top), so just 
        // doing vec seemed simpler, then I added lazy sorting
        // but realized if going that route, should just maintain
        // sort order on input. Long story short there are smarter
        // ways to do this but this is simple and fast, should
        // do this smarter at a later time. 
        let mut sorted = self.vertices.iter().cloned().collect_vec();
        sorted.sort();
        sorted
    }
}

impl PartialEq for ConvexHull {
    fn eq(&self, other: &Self) -> bool {
        self.get_vertices() == other.get_vertices()
    }
}

impl Default for ConvexHull {
    fn default() -> Self {
        let vertices = HashSet::new();
        ConvexHull { vertices }
    }
}


pub trait ConvexHullComputer {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull;
}


#[derive(Default)]
pub struct InteriorPoints; 

impl InteriorPoints {
    pub fn interior_points(&self, polygon: &Polygon) -> HashSet<VertexId> {
        let mut interior_points = HashSet::new();
        let ids = polygon.vertex_ids();

        // Don't be fooled by the runtime here, it's iterating over all
        // permutations, which is n! / (n-4)! = n * (n-1) * (n-2) * (n-3), 
        // so it's still O(n^4), this is just more compact than 4 nested
        // for-loops.
        for perm in ids.into_iter().permutations(4) {
            // TODO instead of unwrap, return result with error
            let p = polygon.get_point(&perm[0]).unwrap();
            let triangle = polygon.get_triangle(&perm[1], &perm[2], &perm[3]).unwrap();
            if triangle.contains(p) {
                interior_points.insert(perm[0]);
            }
        }
        interior_points
    }
}

impl ConvexHullComputer for InteriorPoints {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
        let mut hull = ConvexHull::default();
        // NOTE: This is slow O(n^4) since the interior point 
        // computation being used has that runtime.
        let ids = polygon.vertex_ids_set();
        let interior_ids = self.interior_points(polygon);
        let extreme_ids = &ids - &interior_ids;
        hull.add_vertices(extreme_ids);
        hull
    }
}


#[derive(Default)]
pub struct ExtremeEdges;

impl ExtremeEdges {
    pub fn extreme_edges(&self, polygon: &Polygon) -> Vec<(VertexId, VertexId)> {
        // NOTE: This is O(n^3)
        let mut extreme_edges = Vec::new();
        let ids = polygon.vertex_ids();
    
        for id1 in ids.iter() {
            for id2 in ids.iter() {
                if id2 == id1 {
                    continue;
                }
                // TODO instead of unwrap, return result with error
                let ls = polygon.get_line_segment(id1, id2).unwrap();
                let mut is_extreme = true;
                for id3 in ids.iter() {
                    if id3 == id1 || id3 == id2 {
                        continue;
                    }
                    // TODO instead of unwrap, return result with error
                    let p = polygon.get_point(id3).unwrap();
                    if !p.left_on(&ls) {
                        is_extreme = false;
                        break;
                    }
                }
                if is_extreme {
                    extreme_edges.push((*id1, *id2));
                }
            }
        }
        extreme_edges
    }
}

impl ConvexHullComputer for ExtremeEdges {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
        let mut hull = ConvexHull::default();
        for (id1, id2) in self.extreme_edges(polygon).into_iter() {
            hull.add_vertex(id1);
            hull.add_vertex(id2);
        }
        hull
    }    
}


#[derive(Default)]
pub struct GiftWrapping;

impl ConvexHullComputer for GiftWrapping {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
        // Form a horizontal line terminating at lowest point to start
        let v0 = polygon.rightmost_lowest_vertex();
        let mut p = v0.coords.clone();
        p.x -= 1.0;  // Arbitrary distance
        let mut e = LineSegment::new(&p, &v0.coords);
        let mut v_i = v0;
        
        let mut hull = ConvexHull::default();
        hull.add_vertex(v_i.id);

        // Perform gift-wrapping, using the previous hull edge as a vector to 
        // find the point with the least CCW angle w.r.t. the vector. Connect 
        // that point to the current terminal vertex to form the newest hull 
        // edge. Repeat until we reach the starting vertex again.
        loop {
            let v_min_angle = polygon.vertices()
                .into_iter()
                .filter(|v| v.id != v_i.id)
                .sorted_by_key(|v| (OF(e.angle_to_point(&v.coords)), OF(-v_i.distance_to(v))))
                .dedup_by(|a, b| e.angle_to_point(&a.coords) == e.angle_to_point(&b.coords))
                .collect::<Vec<_>>()[0];

            e = polygon.get_line_segment(&v_i.id, &v_min_angle.id).unwrap();
            v_i = v_min_angle;
            if v_i.id == v0.id {
                break;
            } else {
                hull.add_vertex(v_i.id);
            }
        }
        hull
    }
}


#[derive(Default)]
pub struct QuickHull;

impl ConvexHullComputer for QuickHull {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
        let mut hull = ConvexHull::default();
        let mut stack = Vec::new();

        let x = polygon.lowest_rightmost_vertex().id;
        let y = polygon.highest_leftmost_vertex().id;
        let xy = polygon.get_line_segment(&x, &y).unwrap();
        let s = polygon.vertices()
            .into_iter()
            .filter(|v| v.id != x && v.id != y)
            .collect_vec();

        hull.add_vertex(x);
        hull.add_vertex(y);

        let (s1, s2): (Vec<_>, Vec<_>) = s
            .into_iter()
            .partition(|v| v.right(&xy));

        if !s1.is_empty() { stack.push((x, y, s1)) };
        if !s2.is_empty() { stack.push((y, x, s2)) };

        loop {
            let (a, b, s) = stack.pop().unwrap();
            let ab = polygon.get_line_segment(&a, &b).unwrap();

            let c = s.iter()
                .max_by_key(|v| OF(ab.distance_to_point(&v.coords)))
                .unwrap()
                .id;
            hull.add_vertex(c);

            let ac = polygon.get_line_segment(&a, &c).unwrap();
            let cb = polygon.get_line_segment(&c, &b).unwrap();

            let s1 = s.iter()
                .copied()
                .filter(|v| v.right(&ac))
                .collect_vec();

            let s2 = s.iter()
                .copied()
                .filter(|v| v.right(&cb))
                .collect_vec();

            if !s1.is_empty() { stack.push((a, c, s1)); }
            if !s2.is_empty() { stack.push((c, b, s2)); }
            if stack.is_empty() { break; }
        }
        hull
    }
}


#[derive(Default)]
pub struct GrahamScan;

impl ConvexHullComputer for GrahamScan {
    fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
        let mut stack = Vec::new();
        let mut vertices = polygon.min_angle_sorted_vertices();

        // Add rightmost lowest vertex and the next min-angle vertex
        // to stack to create initial line segment, both guaranteed
        // to be extreme based on vertices being sorted/cleaned
        stack.push(polygon.rightmost_lowest_vertex());
        stack.push(vertices.remove(0));
        
        for v in vertices.iter() {
            // If current vertex is a left turn from current segment off 
            // top of stack, add vertex to incremental hull on stack and 
            // continue to next vertex. Otherwise the current hull on 
            // stack is wrong, continue popping until it's corrected.  
            loop {
                assert!(stack.len() >= 2);
                let v_top = stack[stack.len() - 1];
                let v_prev = stack[stack.len() - 2];
                let ls = polygon.get_line_segment(&v_prev.id, &v_top.id).unwrap();
                if v.left(&ls) {
                    stack.push(v);
                    break;
                } else {
                    stack.pop();
                }
            }
        }
        
        ConvexHull::new(stack.iter().map(|v| v.id).collect())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::{self, *};
    use crate::test_util::*;

    #[apply(extreme_point_cases)]
    // #[rstest]
    // #[case(polygon_1())]
    fn test_convex_hull(
        #[case] 
        case: PolygonTestCase, 
        #[values(ExtremeEdges, GiftWrapping, GrahamScan, InteriorPoints, QuickHull)]
        computer: impl ConvexHullComputer
    ) {
        let hull = computer.convex_hull(&case.polygon);
        let expected_hull = ConvexHull::new(case.metadata.extreme_points);
        assert_eq!(hull, expected_hull);
    }
}
