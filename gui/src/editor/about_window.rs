use super::defines::{hex_code, resource};
use eframe::egui;
use egui_extras::RetainedImage;

#[derive(Default)]
pub struct AboutWindow {}

impl super::WindowBase for AboutWindow {
    fn name(&self) -> String {
        format!("About mgit-gui")
    }

    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {
        let width = 300.0;
        let height = 160.0;
        let screen_rect = eframe.info().window_info.size;
        let default_pos = [
            (screen_rect.x - width) * 0.5,
            (screen_rect.y - height) * 0.5,
        ];

        egui::Window::new(self.name())
            .fixed_pos(default_pos)
            .fixed_size([width, height])
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui);
            });
    }
}

impl super::View for AboutWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
            ui.label("");

            if let Ok(image) = RetainedImage::from_image_bytes("logo128x128.png", resource::LOGO) {
                image.show(ui);
            }

            ui.heading(std::env!("CARGO_PKG_NAME"));

            let version = format!("version {}", std::env!("CARGO_PKG_VERSION"));
            ui.label(version);

            ui.hyperlink_to(
                format!("{} mgit on github", hex_code::GIT),
                std::env!("CARGO_PKG_REPOSITORY"),
            );
            ui.label("");
        });
    }
}
