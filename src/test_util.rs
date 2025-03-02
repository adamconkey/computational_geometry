use rstest::fixture;
use rstest_reuse::{self, *};    
use serde::Deserialize;
use std::{collections::HashSet, fs, path::PathBuf};

use crate::{
    error::FileError, 
    polygon::Polygon,
    util::load_polygon, 
    vertex::VertexId
};


#[derive(Deserialize)]
pub struct PolygonMetadata {
    pub area: f64,
    pub extreme_points: HashSet<VertexId>,
    pub interior_points: HashSet<VertexId>,
    pub num_edges: usize,
    pub num_triangles: usize,
    pub num_vertices: usize,
}

pub struct PolygonTestCase {
    pub polygon: Polygon,
    pub metadata: PolygonMetadata,
}

impl PolygonTestCase {
    fn new(polygon: Polygon, metadata: PolygonMetadata) -> Self {
        PolygonTestCase { polygon, metadata }
    }
}

fn load_metadata(name: &str, folder: &str) -> Result<PolygonMetadata, FileError> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("polygons");
    path.push(folder);
    path.push(format!("{}.meta.json", name));
    let metadata_str: String = fs::read_to_string(path)?;
    let metadata = serde_json::from_str(&metadata_str)?;
    Ok(metadata)
}

#[macro_export]
macro_rules! polygon_fixture {
    ($name:ident, $folder:expr) => {
        #[fixture]
        pub fn $name() -> PolygonTestCase {
            PolygonTestCase::new(
                load_polygon(stringify!($name), stringify!($folder)).unwrap(),
                load_metadata(stringify!($name), stringify!($folder)).unwrap(),
            )
        }
    };
}

polygon_fixture!(polygon_1, custom);
polygon_fixture!(polygon_2, custom);
polygon_fixture!(right_triangle, custom);
polygon_fixture!(square_4x4, custom);

polygon_fixture!(eberly_10, interesting_polygon_archive);
polygon_fixture!(eberly_14, interesting_polygon_archive);
polygon_fixture!(elgindy_1, interesting_polygon_archive);
polygon_fixture!(gray_embroidery, interesting_polygon_archive);
polygon_fixture!(held_1, interesting_polygon_archive);
polygon_fixture!(held_3, interesting_polygon_archive);
polygon_fixture!(held_12, interesting_polygon_archive);
polygon_fixture!(held_7a, interesting_polygon_archive);
polygon_fixture!(held_7b, interesting_polygon_archive);
polygon_fixture!(held_7c, interesting_polygon_archive);
polygon_fixture!(held_7d, interesting_polygon_archive);
polygon_fixture!(mapbox_building, interesting_polygon_archive);
polygon_fixture!(mapbox_dude, interesting_polygon_archive);
polygon_fixture!(matisse_alga, interesting_polygon_archive);
polygon_fixture!(matisse_blue, interesting_polygon_archive);
polygon_fixture!(matisse_icarus, interesting_polygon_archive);
polygon_fixture!(matisse_nuit, interesting_polygon_archive);
polygon_fixture!(mei_2, interesting_polygon_archive);
polygon_fixture!(mei_3, interesting_polygon_archive);
polygon_fixture!(mei_4, interesting_polygon_archive);
polygon_fixture!(mei_5, interesting_polygon_archive);
polygon_fixture!(mei_6, interesting_polygon_archive);
polygon_fixture!(meisters_3, interesting_polygon_archive);
polygon_fixture!(misc_discobolus, interesting_polygon_archive);
polygon_fixture!(misc_fu, interesting_polygon_archive);
polygon_fixture!(seidel_3, interesting_polygon_archive);
polygon_fixture!(skimage_horse, interesting_polygon_archive);
polygon_fixture!(toussaint_1a, interesting_polygon_archive);

#[template]
#[rstest]
#[case::right_triangle(right_triangle())]
#[case::square_4x4(square_4x4())]
#[case::polygon_1(polygon_1())]
#[case::polygon_2(polygon_2())]
#[case::eberly_10(eberly_10())]
#[case::eberly_14(eberly_14())]
#[case::elgindy_1(elgindy_1())]
#[case::gray_embroidery(gray_embroidery())]
#[case::held_1(held_1())]
#[case::held_3(held_3())]
#[case::held_12(held_12())]
#[case::held_7a(held_7a())]
#[case::held_7b(held_7b())]
#[case::held_7c(held_7c())]
#[case::held_7d(held_7d())]
#[case::mapbox_building(mapbox_building())]
#[case::mapbox_dude(mapbox_dude())]
#[case::matisse_alga(matisse_alga())]
#[case::matisse_blue(matisse_blue())]
#[case::matisse_icarus(matisse_icarus())]
#[case::matisse_nuit(matisse_nuit())]
#[case::mei_2(mei_2())]
#[case::mei_3(mei_3())]
#[case::mei_4(mei_4())]
#[case::mei_5(mei_5())]
#[case::mei_6(mei_6())]
#[case::meisters_3(meisters_3())]
#[case::misc_discobolus(misc_discobolus())]
#[case::misc_fu(misc_fu())]
#[case::seidel_3(seidel_3())]
#[case::skimage_horse(skimage_horse())]
#[case::toussaint_1a(toussaint_1a())]
pub fn all_polygons(#[case] case: PolygonTestCase) {}

#[template]
#[rstest]
#[case::polygon_1(polygon_1())]
#[case::polygon_2(polygon_2())]
#[case::right_triangle(right_triangle())]
#[case::square_4x4(square_4x4())]
pub fn all_custom_polygons(#[case] case: PolygonTestCase) {}

#[template]
#[apply(all_custom_polygons)]
#[case::eberly_10(eberly_10())]
#[case::eberly_14(eberly_14())]
#[case::elgindy_1(elgindy_1())]
#[case::gray_embroidery(gray_embroidery())]
#[case::held_1(held_1())]
#[case::held_12(held_12())]
#[case::held_3(held_3())]
// TODO can add more test cases here, but I've just been going through
// and manually verifying they're correct. Some have so many vertices
// though it's not practical
pub fn extreme_point_cases(#[case] case: PolygonTestCase) {}
