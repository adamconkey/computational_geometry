use egui::Response;
use egui_plot::{
    CoordinatesFormatter, Corner, Line, 
    Plot, Points, Polygon as PlotPolygon
};
use std::collections::HashMap;
use std::fmt;

use computational_geometry::{
    point::Point,
    polygon::Polygon,
};

use crate::app::RESULT_DIR;


#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum Visualization {
    Polygon,
    Triangulation,
}

impl fmt::Display for Visualization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


//#[derive(PartialEq)]
pub struct PolygonVisualizer {
    points: HashMap<String, Vec<[f64; 2]>>,
    triangulations: HashMap<String, Vec<(Point, Point, Point)>>,
    line_width: f32,
    point_radius: f32,
    selected_visualization: Visualization,
}

impl Default for PolygonVisualizer {
    fn default() -> Self {
        let mut points = HashMap::new();
        let mut triangulations = HashMap::new();
        
        for file in RESULT_DIR.files() {
            let stem = String::from(file.path().file_stem().unwrap().to_str().unwrap());
            let contents = String::from(file.contents_utf8().unwrap());
            let polygon_points: Vec<_> = serde_json::from_str::<Vec<Point>>(&contents)
                .unwrap();

            let mut plot_points: Vec<_> = polygon_points
                .iter()
                .map(|p: &Point| [p.x, p.y])
                .collect();
            // Pushing first to end so it closes the chain, probably
            // only want to do this for line points since it
            // duplicates a vertex
            plot_points.push(*plot_points.first().unwrap());
            points.insert(stem.clone(), plot_points);

            let polygon = Polygon::new(polygon_points);
            let triangulation_points = polygon.triangulation()
                .to_points();
            triangulations.insert(stem.clone(), triangulation_points);
        }

        Self { 
            points,
            triangulations,
            line_width: 4.0, 
            point_radius: 8.0, 
            selected_visualization: Visualization::Polygon,
        }
    }
}

impl PolygonVisualizer {
    pub fn ui(&mut self, ui: &mut egui::Ui, name: &String) -> Response {
        
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.selected_visualization, 
                Visualization::Polygon,
                Visualization::Polygon.to_string(),
            );
            ui.selectable_value(
                &mut self.selected_visualization, 
                Visualization::Triangulation,
                Visualization::Triangulation.to_string(),
            );
        });
        ui.separator();
        
        match self.selected_visualization {
            Visualization::Polygon => {
                self.draw_polygon(ui, name)
            }
            Visualization::Triangulation => {
                self.draw_triangulation(ui, name)
            }
        }
    }

    fn draw_polygon(&self, ui: &mut egui::Ui, name: &String) -> Response {
        let plot = self.create_plot();
        let line = self.create_line(name);
        let points = self.create_points(name);

        plot.show(ui, |plot_ui| {
            plot_ui.line(line);
            plot_ui.points(points);
        }).response
    }

    fn draw_triangulation(&self, ui: &mut egui::Ui, name: &String) -> Response {
        let plot = self.create_plot();
        let triangulation = self.triangulations.get(name).unwrap();
        let triangles: Vec<_> = triangulation
            .iter()
            .map(|(p1, p2, p3)|
                PlotPolygon::new(
                    vec![[p1.x, p1.y], [p2.x, p2.y], [p3.x, p3.y]]
                )
        ).collect();

        plot.show(ui, |plot_ui| {
            for triangle in triangles.into_iter() {
                plot_ui.polygon(triangle);
            }
        }).response
    }

    fn create_plot(&self) -> Plot<'_> {
        Plot::new("polygon_visualizer")
            .show_axes(true)
            .show_grid(true)
            .view_aspect(1.0)
            .data_aspect(1.0)
            .coordinates_formatter(
                Corner::LeftBottom, 
                CoordinatesFormatter::default()
            )
    }

    fn create_line(&self, name: &String) -> Line {
        let points = self.points.get(name).unwrap();
        Line::new(points.clone())
            .width(self.line_width)
    }

    fn create_points(&self, name: &String) -> Points {
        let points = self.points.get(name).unwrap();
        Points::new(points.clone())
            .radius(self.point_radius)
    }
}