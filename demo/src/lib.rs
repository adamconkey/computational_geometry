//! Demo code for the [`egui_plot`], hosted at <https://emilk.github.io/egui_plot/>.
//!
//! Each push to `main` re-deploys the demo.

#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod plot_demo;

pub use app::TemplateApp;

/// Create a [`Hyperlink`](egui::Hyperlink) to this egui source code file on github.
#[macro_export]
macro_rules! egui_github_link_file {
    () => {
        $crate::egui_github_link_file!("(source code)")
    };
    ($label: expr) => {
        egui::github_link_file!(
            "https://github.com/emilk/egui_plot/blob/main/",
            egui::RichText::new($label).small()
        )
    };
}

/// Create a [`Hyperlink`](egui::Hyperlink) to this egui source code file and line on github.
#[macro_export]
macro_rules! egui_github_link_file_line {
    () => {
        $crate::egui_github_link_file_line!("(source code)")
    };
    ($label: expr) => {
        egui::github_link_file_line!(
            "https://github.com/emilk/egui_plot/blob/main/",
            egui::RichText::new($label).small()
        )
    };
}
