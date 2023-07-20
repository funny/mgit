use eframe::egui;
use eframe::egui::Vec2;

use crate::editor::repositories::create_layout_job;
use crate::utils::defines::text_color;

#[derive(Default)]
pub struct ErrorWindow {
    content: String,
}
impl ErrorWindow {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl super::WindowBase for ErrorWindow {
    fn name(&self) -> String {
        "Errors".to_string()
    }

    fn width(&self) -> f32 {
        300.0
    }

    fn height(&self) -> f32 {
        100.0
    }

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2] {
        [
            (screen_rect.x - self.width()) * 0.5,
            (screen_rect.y - self.height()) * 0.5,
        ]
    }
}

impl super::View for ErrorWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        use super::WindowBase;
        ui.set_min_size(egui::vec2(self.width(), self.height()));
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.add_space(5.0);

            let job = create_layout_job(self.content.clone(), text_color::RED);
            ui.label(job);

            ui.add_space(5.0);

            ui.hyperlink_to("git official website".to_string(), "https://git-scm.com/");
        });
    }
}
