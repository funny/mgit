// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::NativeOptions;

use app::GuiApp;

use crate::ui::style::load_icon;
use crate::ui::style::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::utils::logger::init_log;

pub(crate) mod app;
pub(crate) mod configs;
pub(crate) mod ui;
pub(crate) mod utils;

fn main() -> eframe::Result {
    let _guard = init_log();

    // Log mgit-gui version at startup
    let version = std::env!("CARGO_PKG_VERSION");
    tracing::info!(
        name = std::env!("CARGO_PKG_NAME"),
        version = version,
        "mgit_gui_started"
    );

    let native_options = NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([DEFAULT_WIDTH, DEFAULT_HEIGHT])
            .with_min_inner_size([666.0, 480.0])
            .with_decorations(true)
            .with_transparent(false)
            .with_resizable(true)
            .with_icon(load_icon())
            .with_drag_and_drop(true),
        ..NativeOptions::default()
    };

    eframe::run_native(
        &format!(
            "{} {}",
            std::env!("CARGO_PKG_NAME"),
            std::env!("CARGO_PKG_VERSION")
        ),
        native_options,
        Box::new(|cc| Ok(Box::new(GuiApp::new(cc)))),
    )
}
