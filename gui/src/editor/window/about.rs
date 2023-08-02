use eframe::egui;
use eframe::egui::Vec2;
use egui_extras::RetainedImage;

use crate::utils::defines::{hex_code, resource};

#[derive(Default)]
pub struct AboutWindow {
    pub mgit_version: String,
}

impl super::WindowBase for AboutWindow {
    fn name(&self) -> String {
        "About mgit-gui".to_string()
    }

    fn width(&self) -> f32 {
        300.0
    }

    fn height(&self) -> f32 {
        160.0
    }

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2] {
        [
            (screen_rect.x - self.width()) * 0.5,
            (screen_rect.y - self.height()) * 0.5,
        ]
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

            // mgit gui version
            let text = format!("mgit-gui v{}", std::env!("CARGO_PKG_VERSION"));
            ui.label(text);

            ui.hyperlink_to(
                format!("{} mgit on github", hex_code::GIT),
                std::env!("CARGO_PKG_REPOSITORY"),
            );
            ui.label("");
        });
    }
}
