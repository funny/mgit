// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::logger::init_log;
use editor::{
    defines::{DEFAULT_HEIGHT, DEFAULT_WIDTH},
    load_icon, App,
};
use eframe::NativeOptions;

mod editor;
mod logger;
pub(crate) mod progress;

fn main() {
    init_log();
    let native_options = NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some([DEFAULT_WIDTH, DEFAULT_HEIGHT].into()),
        min_window_size: Some(eframe::egui::vec2(666.0, 480.0)),
        decorated: true,
        transparent: true,
        resizable: true,
        icon_data: Some(load_icon()),
        ..NativeOptions::default()
    };

    eframe::run_native(
        &format!(
            "{} {}",
            std::env!("CARGO_PKG_NAME"),
            std::env!("CARGO_PKG_VERSION")
        ),
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
