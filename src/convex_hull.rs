use itertools::Itertools;
use ordered_float::OrderedFloat as OF;
use std::{cmp::Reverse, collections::HashSet};

use crate::{
    line_segment::LineSegment, 
    polygon::Polygon, 
    vertex::VertexId
};


// #[derive(Debug)]
// pub struct ConvexHull {
//     vertices: HashSet<VertexId>,
// }

// impl ConvexHull {
//     pub fn new(vertices: HashSet<VertexId>) -> Self {
//         ConvexHull { vertices }
//     }

//     pub fn add_vertex(&mut self, vertex_id: VertexId) {
//         self.vertices.insert(vertex_id);
//     }

//     pub fn add_vertices(&mut self, vertices: impl IntoIterator<Item = VertexId>) {
//         for v in vertices {
//             self.add_vertex(v);
//         }
//     }

//     pub fn get_vertices(&self) -> Vec<VertexId> {
//         // TODO this is a little silly to sort on every retrieval 
//         // especially if nothing changes. I originally had a 
//         // min-heap but found you still had to clone/sort to get 
//         // the whole vec (not just peek/pop from top), so just 
//         // doing vec seemed simpler, then I added lazy sorting
//         // but realized if going that route, should just maintain
//         // sort order on input. Long story short there are smarter
//         // ways to do this but this is simple and fast, should
//         // do this smarter at a later time. 
//         let mut sorted = self.vertices.iter().cloned().collect_vec();
//         sorted.sort();
//         sorted
//     }
// }

// impl PartialEq for ConvexHull {
//     fn eq(&self, other: &Self) -> bool {
//         self.get_vertices() == other.get_vertices()
//     }
// }

// impl Default for ConvexHull {
//     fn default() -> Self {
//         let vertices = HashSet::new();
//         ConvexHull { vertices }
//     }
// }


pub trait ConvexHullComputer {
    fn convex_hull(&self, polygon: &Polygon) -> Polygon;
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
    fn convex_hull(&self, polygon: &Polygon) -> Polygon {
        // NOTE: This is slow O(n^4) since the interior point 
        // computation being used has that runtime.
        let interior_ids = self.interior_points(polygon);
        let hull_ids = &polygon.vertex_ids_set() - &interior_ids;
        let mut hull_vertices = polygon
            .get_vertices(hull_ids)
            .into_iter()
            .cloned()
            .collect_vec();
        hull_vertices.sort_by_key(|v| v.id);
        Polygon::from_vertices(hull_vertices)
    }
}


#[derive(Default)]
pub struct ExtremeEdges;

impl ExtremeEdges {
    pub fn extreme_edges(&self, polygon: &Polygon) -> Vec<(VertexId, VertexId)> {
        // NOTE: This is O(n^3)
        let mut extreme_edges = Vec::new();
        let ids = polygon.vertex_ids();
    
        for perm in ids.iter().permutations(2) {
            // TODO instead of unwrap, return result with error
            let ls = polygon.get_line_segment(perm[0], perm[1]).unwrap();
            let mut is_extreme = true;
            for id3 in ids.iter().filter(|id| !perm.contains(id)) {
                // TODO instead of unwrap, return result with error
                let p = polygon.get_point(id3).unwrap();
                if !p.left_on(&ls) {
                    is_extreme = false;
                    break;
                }
            }
            if is_extreme {
                extreme_edges.push((*perm[0], *perm[1]));
            }
        }

        // Have to do this cleaning step to account for collinear points in
        // the edge chain. Note the edge chain as-is could have collinear
        // points even if the polygon itself does not have collinear points.
        // If there's a chain xyz, this procedure will keep xz being the two
        // points furthest from each other no matter how many collinear
        // points exist between those two.
        self.remove_collinear(&mut extreme_edges, polygon);
        extreme_edges
    }

    fn remove_collinear(&self, edges: &mut Vec<(VertexId, VertexId)>, p: &Polygon) {
        edges.sort_by_key(|(id1, id2)| (*id1, Reverse(OF(p.distance_between(id1, id2)))));
        edges.dedup_by(|a, b| a.0 == b.0);
        edges.sort_by_key(|(id1, id2)| (*id2, Reverse(OF(p.distance_between(id1, id2)))));
        edges.dedup_by(|a, b| a.1 == b.1);
    }
}

impl ConvexHullComputer for ExtremeEdges {
    fn convex_hull(&self, polygon: &Polygon) -> Polygon {
        let mut hull_ids = HashSet::new();
        for (id1, id2) in self.extreme_edges(polygon).into_iter() {
            hull_ids.insert(id1);
            hull_ids.insert(id2);
        }
        let mut hull_vertices = polygon
            .get_vertices(hull_ids)
            .into_iter()
            .cloned()
            .collect_vec();
        hull_vertices.sort_by_key(|v| v.id);
        Polygon::from_vertices(hull_vertices)
    }    
}


#[derive(Default)]
pub struct GiftWrapping;

impl ConvexHullComputer for GiftWrapping {
    fn convex_hull(&self, polygon: &Polygon) -> Polygon {
        // Form a horizontal line terminating at lowest point to start
        let v0 = polygon.rightmost_lowest_vertex();
        let mut p = v0.coords.clone();
        p.x -= 1.0;  // Arbitrary distance
        let mut e = LineSegment::new(&p, &v0.coords);
        let mut v_i = v0;
        
        let mut hull_ids = HashSet::new();
        hull_ids.insert(v_i.id);

        // Perform gift-wrapping, using the previous hull edge as a vector to 
        // find the point with the least CCW angle w.r.t. the vector. Connect 
        // that point to the current terminal vertex to form the newest hull 
        // edge. Repeat until we reach the starting vertex again.
        loop {
            let v_min_angle = polygon.vertices()
                .into_iter()
                .filter(|v| v.id != v_i.id)
                .sorted_by_key(|v| (OF(e.angle_to_point(&v.coords)), Reverse(OF(v_i.distance_to(v)))))
                .dedup_by(|a, b| e.angle_to_point(&a.coords) == e.angle_to_point(&b.coords))
                .collect::<Vec<_>>()[0];

            e = polygon.get_line_segment(&v_i.id, &v_min_angle.id).unwrap();
            v_i = v_min_angle;
            if v_i.id == v0.id {
                break;
            } else {
                hull_ids.insert(v_i.id);
            }
        }
        let mut hull_vertices = polygon
            .get_vertices(hull_ids)
            .into_iter()
            .cloned()
            .collect_vec();
        hull_vertices.sort_by_key(|v| v.id);
        Polygon::from_vertices(hull_vertices)
    }
}


#[derive(Default)]
pub struct QuickHull;

impl ConvexHullComputer for QuickHull {
    fn convex_hull(&self, polygon: &Polygon) -> Polygon {
        let mut hull_ids = HashSet::new();
        let mut stack = Vec::new();

        let x = polygon.lowest_rightmost_vertex().id;
        let y = polygon.highest_leftmost_vertex().id;
        let xy = polygon.get_line_segment(&x, &y).unwrap();
        let s = polygon.vertices()
            .into_iter()
            .filter(|v| v.id != x && v.id != y)
            .collect_vec();

        hull_ids.insert(x);
        hull_ids.insert(y);

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
            hull_ids.insert(c);

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
        let mut hull_vertices = polygon
            .get_vertices(hull_ids)
            .into_iter()
            .cloned()
            .collect_vec();
        hull_vertices.sort_by_key(|v| v.id);
        Polygon::from_vertices(hull_vertices)
    }
}


#[derive(Default)]
pub struct GrahamScan;

impl ConvexHullComputer for GrahamScan {
    fn convex_hull(&self, polygon: &Polygon) -> Polygon {
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
        
        let hull_ids = stack.iter().map(|v| v.id).collect_vec();
        let mut hull_vertices = polygon
            .get_vertices(hull_ids)
            .into_iter()
            .cloned()
            .collect_vec();
        hull_vertices.sort_by_key(|v| v.id);
        Polygon::from_vertices(hull_vertices)
    }
}


// #[derive(Default)]
// pub struct Incremental;

// impl ConvexHullComputer for Incremental {
    
//     // TODO it's at this stage that it may make sense to consider
//     // making the ConvexHull just, a Polygon. Not sure yet how
//     // difficult that would be yet but it could make a lot of
//     // sense, since now I'll need to be updating refs and
//     // maintaining a sorted vec of vertices, which is all stuff
//     // offered already by Polygon. Plus you'd get a lot for free
//     // with whatever else would be called on Polygon that also
//     // applies to hull. 
    
//     fn convex_hull(&self, polygon: &Polygon) -> ConvexHull {
//         let mut vertices = polygon.vertices();
//         vertices.sort_by_key(|v| OF(v.coords.x));

//         let mut hull = ConvexHull::default();

//         // TODO populate hull with first 3 vertices (triangle)

//         // TODO iterate over vertices, for each one, find the
//         // upper and lower tangents from the point to the hull.
//         // I think since they're sorted left-to-right, should
//         // able to take uppermost and lowermost vertices of
//         // hull?

//         // TODO update hull chain prev/next refs to form new
//         // hull.

//         hull
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use rstest_reuse::{self, *};
    use crate::test_util::*;

    #[apply(extreme_point_cases)]
    fn test_convex_hull(
        #[case] 
        case: PolygonTestCase, 
        #[values(ExtremeEdges, GiftWrapping, GrahamScan, InteriorPoints, QuickHull)]
        computer: impl ConvexHullComputer
    ) {
        let hull = computer.convex_hull(&case.polygon);
        assert_eq!(hull.get_vertex_ids(), case.metadata.extreme_points);
    }
}
