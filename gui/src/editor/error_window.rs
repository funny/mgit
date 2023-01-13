use super::{create_layout_job, defines::text_color, defines::MGIT_VERSION};
use eframe::egui;

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
        format!("Errors")
    }

    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {
        let width = 300.0;
        let height = 100.0;
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
                ui.set_min_size(egui::vec2(width, height));
                use super::View;
                self.ui(ui);
            });
    }
}

impl super::View for ErrorWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.add_space(5.0);

            let job = create_layout_job(self.content.clone(), text_color::RED);
            ui.label(job);

            ui.add_space(5.0);

            ui.hyperlink_to(
                format!("mgit {} on github", MGIT_VERSION),
                format!("https://github.com/funny/mgit/releases/tag/{}", MGIT_VERSION)
            );
            ui.hyperlink_to(
                format!("git official website"),
                "https://git-scm.com/"
            );
        });
    }
}
