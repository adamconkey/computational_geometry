use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use geometer::{
    convex_hull::{ConvexHullComputer, ExtremeEdges, GiftWrapping, InteriorPoints, QuickHull}, 
    util::polygon_map_by_num_vertices
};


fn benchmark_convex_hull(c: &mut Criterion) {
    let polygon_map = polygon_map_by_num_vertices(200usize).unwrap();
    let mut group = c.benchmark_group("Convex Hull");
    group.sample_size(10);

    for (name, polygon) in polygon_map.iter() {
        group.bench_with_input(
            BenchmarkId::new("interior_points", name),
            polygon, 
            |b, polygon| b.iter(|| InteriorPoints.convex_hull(polygon))
        );
        group.bench_with_input(
            BenchmarkId::new("extreme_edges", name),
            polygon, 
            |b, polygon| b.iter(|| ExtremeEdges.convex_hull(polygon))
        );
        group.bench_with_input(
            BenchmarkId::new("gift_wrapping", name),
            polygon, 
            |b, polygon| b.iter(|| GiftWrapping.convex_hull(polygon))
        );
        group.bench_with_input(
            BenchmarkId::new("quick_hull", name),
            polygon, 
            |b, polygon| b.iter(|| QuickHull.convex_hull(polygon))
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark_convex_hull);
criterion_main!(benches);