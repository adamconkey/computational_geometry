#[cfg(test)]
// This is currently the strictest I could be across ALL
// tests that use float asserts. Most tests satisfy a
// lot more precision so not sure if this is the best
// way to go about this, but for now this shows the
// empirical precision limit on the entire test suite
const F64_ASSERT_PRECISION: f64 = 1e-4f64;

pub mod bounding_box;
pub mod convex_hull;
pub mod error;
pub mod line_segment;
pub mod point;
pub mod polygon;
pub mod triangle;
pub mod triangulation;
pub mod util;
pub mod vector;
pub mod vertex;

#[cfg(test)]
pub mod test_util;