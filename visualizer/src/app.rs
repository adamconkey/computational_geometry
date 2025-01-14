use include_dir::{include_dir, Dir};

pub const RESULT_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/polygons");


#[derive(Default)]
pub struct TemplateApp {
    visualizer: crate::polygon_visualizer::PolygonVisualizer,
    filenames: Vec<String>,
    selected_polygon: String,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new() -> Self {
        let filenames: Vec<_> = RESULT_DIR
            .files()
            .map(|f| String::from(f.path().file_stem().unwrap().to_str().unwrap()))
            .collect();
        let selected_polygon = filenames[0].clone();
        
        Self { 
            visualizer: crate::polygon_visualizer::PolygonVisualizer::default(), 
            filenames, 
            selected_polygon,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        egui::TopBottomPanel::bottom("theme_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("polygon_browser")
            .resizable(true)
            .default_width(100.0)
            .min_width(100.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Polygons");
                });
                ui.separator();

                for name in self.filenames.iter() {
                    ui.selectable_value(
                        &mut self.selected_polygon, 
                        name.to_string(), 
                        name
                    );
                }
            });

        // This was needed to workaround some artifacts in the plot for
        // highly obtuse triangles in triangulations. It's possible it
        // causes other problems down the road, so can try setting to
        // true if other things look strange.
        // https://github.com/adamconkey/computational_geometry/issues/17
        ctx.tessellation_options_mut(|to| {
            to.feathering = false;
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.visualizer.ui(ui, &self.selected_polygon);
        });
    }
}
