// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::NativeOptions;

use editor::Editor;

use crate::editor::misc::load_icon;
use crate::utils::defines::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::utils::logger::init_log;

pub(crate) mod editor;
pub(crate) mod toml_settings;
pub(crate) mod utils;

fn main() {
    init_log();

    let native_options = NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some([DEFAULT_WIDTH, DEFAULT_HEIGHT].into()),
        min_window_size: Some(eframe::egui::vec2(666.0, 480.0)),
        decorated: true,
        transparent: false,
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
        Box::new(|cc| Box::new(Editor::new(cc))),
    );
}
