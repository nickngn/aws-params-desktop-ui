#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod aws;
mod bridge;
mod state;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([700.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "AWS Param UI",
        options,
        Box::new(|cc| Ok(Box::new(app::AwsParamApp::new(cc)))),
    )
}
