use clap::{Parser, ValueEnum};
use geometer::convex_hull::{ConvexHullComputer, QuickHull};
use geometer::error::FileError;
use itertools::Itertools;
use random_color::RandomColor;

use geometer::polygon::Polygon;
use geometer::triangulation::{EarClipping, Triangulation, TriangulationComputer};
use geometer::util::load_polygon;


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Visualization {
    ConvexHull,
    Triangulation,
}


/// Visualize polygons and algorithms using Rerun.io``
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Folder containing polygon file
    #[arg(short, long, default_value = "interesting_polygon_archive")]
    folder: String,

    /// Polygon name to visualize (without the .json extension)
    #[arg(short, long, default_value = "skimage_horse")]
    polygon: String,

    /// Type of visualization to generate
    #[arg(short, long, value_enum)]
    visualization: Visualization,
}


#[derive(Debug)]
pub enum VisualizationError {
    File(FileError),
    Rerun(rerun::RecordingStreamError),
}

impl From<FileError> for VisualizationError {
    fn from(value: FileError) -> Self {
        VisualizationError::File(value)
    }
}

impl From<rerun::RecordingStreamError> for VisualizationError {
    fn from(value: rerun::RecordingStreamError) -> Self {
        VisualizationError::Rerun(value)
    }
}


pub struct RerunVisualizer {
    rec: rerun::RecordingStream,
}


impl RerunVisualizer {
    pub fn new(name: String) -> Result<Self, VisualizationError> {
        let rec = rerun::RecordingStreamBuilder::new(name).connect_tcp()?;
        Ok(RerunVisualizer { rec })
    }

    pub fn visualize_polygon(
        &self, 
        polygon: &Polygon, 
        name: &String, 
        vertex_radius: f32
    ) -> Result<(), VisualizationError> {
        let vertex_color = RandomColor::new().to_rgb_array();
        self.rec.log(
            format!("{}/vertices", name),
            &self.polygon_to_rerun_points(polygon)
                .with_radii([vertex_radius])
                .with_colors([vertex_color])
        )?;

        let edge_color = RandomColor::new().to_rgb_array();
        self.rec.log(
            format!("{}/edges", name),
            &self.polygon_to_rerun_edges(polygon)
                .with_colors([edge_color])
        )?;
        
        Ok(())
    }

    pub fn visualize_triangulation(&self, polygon: &Polygon, name: &String) -> Result<(), VisualizationError> {
        let name = format!("{name}/triangulation");
        let triangulation = EarClipping.triangulation(polygon);
        let rerun_meshes = self.triangulation_to_rerun_meshes(&triangulation, polygon);

        let _ = self.visualize_polygon(polygon, &name, 5.0);
        
        for (i, mesh) in rerun_meshes.iter().enumerate() {
            self.rec.log(
                format!("{}/triangle_{}", &name, i),
                mesh
            )?;
        }

        Ok(())
    }

    pub fn visualize_convex_hull(&self, polygon: &Polygon, name: &String) -> Result<(), VisualizationError> {    
        let _ = self.visualize_polygon(polygon, &format!("{name}/polygon"), 5.0);
        
        let hull = QuickHull.convex_hull(polygon);
        let points = hull.get_vertices()
            .iter()
            .map(|id| polygon.get_vertex(id).unwrap().coords.clone())
            .collect_vec();
        let hull_polygon = Polygon::new(points);

        let _ = self.visualize_polygon(&hull_polygon, &format!("{name}/convex_hull"), 10.0);

        Ok(())
    }

    fn polygon_to_rerun_points(&self, polygon: &Polygon) -> rerun::Points3D {
        rerun::Points3D::new(
            polygon.sorted_points()
                .into_iter()
                .map(|p| (p.x as f32, p.y as f32, 0.0))
        )
    }
    
    fn polygon_to_rerun_edges(&self, polygon: &Polygon) -> rerun::LineStrips3D {
        let mut edge_points: Vec<_> = polygon.sorted_points()
            .into_iter()
            .map(|p| (p.x as f32, p.y as f32, 0.0))
            .collect();
        edge_points.push(edge_points[0]);
        rerun::LineStrips3D::new([edge_points])
    }
    
    fn triangulation_to_rerun_meshes(&self, triangulation: &Triangulation, polygon: &Polygon) -> Vec<rerun::Mesh3D> {
        let mut meshes = Vec::new();
        for ids in triangulation.iter() {
            let color = RandomColor::new().to_rgb_array();
            let t = polygon.get_triangle(&ids.0, &ids.1, &ids.2).unwrap();
            let points = [
                [t.p1.x as f32, t.p1.y as f32, 0.0], 
                [t.p2.x as f32, t.p2.y as f32, 0.0], 
                [t.p3.x as f32, t.p3.y as f32, 0.0]
            ]; 
            let mesh = rerun::Mesh3D::new(points)
                .with_vertex_colors([color, color, color]);
            meshes.push(mesh);
        }
        meshes
    }
}


fn main() -> Result<(), VisualizationError> {
    let args = Args::parse();
    let visualizer = RerunVisualizer::new("Geometer".to_string());

    let polygon = load_polygon(&args.polygon, &args.folder)?;
    let name = format!("{}/{}", args.polygon, args.folder);

    match args.visualization {
        Visualization::ConvexHull => visualizer?.visualize_convex_hull(&polygon, &name)?,
        Visualization::Triangulation => visualizer?.visualize_triangulation(&polygon, &name)?,
    };
    
    Ok(())
}