// hide console window on Windows in release
#![windows_subsystem = "windows"]
use editor::{
    defines::{DEFAULT_HEIGHT, DEFAULT_WIDTH},
    load_icon, App,
};

mod editor;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = true;
    native_options.initial_window_size = Some([DEFAULT_WIDTH, DEFAULT_HEIGHT].into());
    native_options.min_window_size = Some(eframe::egui::vec2(666.0, 480.0));
    // native_options.default_theme = eframe::Theme::Dark;
    native_options.decorated = true;
    native_options.transparent = true;
    native_options.resizable = true;
    native_options.icon_data = Some(load_icon());

    eframe::run_native(
        &format!(
            "{} {}",
            std::env!("CARGO_PKG_NAME").to_string(),
            std::env!("CARGO_PKG_VERSION").to_string()
        ),
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    );
}
