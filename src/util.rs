use itertools::Itertools;
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};
use walkdir::WalkDir;

use crate::error::FileError;
use crate::polygon::Polygon;


pub fn load_polygon(name: &str, folder: &str) -> Result<Polygon, FileError> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("polygons");
    path.push(folder);
    path.push(format!("{}.json", name));
    Polygon::from_json(path)
}


pub fn polygon_map_by_num_vertices(vertex_limit: usize) -> Result<HashMap<usize, Polygon>, FileError> {
    let mut map = HashMap::new();
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.push("polygons");
    let paths = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| p.is_file())
        .filter(|p| p.extension() == Some(OsStr::new("json")))
        // Remove .meta.json files
        .filter(|p| p.with_extension("").extension() != Some(OsStr::new("meta")));
    for path in paths.sorted() {
        let p = Polygon::from_json(path)?;
        if p.num_vertices() <= vertex_limit {
            map.insert(p.num_vertices(), p);
        }
    }
    Ok(map)
}
