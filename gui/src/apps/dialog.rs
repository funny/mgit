use super::View;
use eframe::{egui, epaint::text::LayoutJob};

#[derive(Default)]
pub struct Dialog {
    name: String,
    content: String,
    is_ok: Option<bool>,
}

impl super::DialogBase for Dialog {
    fn create(name: String, content: String) -> Self {
        Self {
            name,
            content,
            is_ok: None,
        }
    }

    fn is_ok(&self) -> bool {
        match self.is_ok {
            Some(r) => r,
            _ => false,
        }
    }
}

impl super::WindowBase for Dialog {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        let width = 300.0;
        let height = 160.0;
        let screen_rect = ctx.used_size();
        let default_pos = [
            (screen_rect.x - width) * 0.5,
            (screen_rect.y - height) * 0.5,
        ];
        self.is_ok = None;

        egui::Window::new(&self.name)
            .default_pos(default_pos)
            .fixed_size([width, height])
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui);
            });

        if self.is_ok.is_some() {
            *open = false;
        }
    }
}

impl super::View for Dialog {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 20.0);
            ui.label("");
            ui.label(&self.content);

            // cancel_ok_options
            ui.horizontal(|ui| {
                // ok button
                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(80.0, 0.0),
                    egui::vec2(60.0, 20.0),
                );
                let mut job = LayoutJob::default();
                job.append(
                    "Ok",
                    0.0,
                    egui::TextFormat {
                        color: super::defines::text_color::GREEN,
                        ..Default::default()
                    },
                );
                let ok_btn = egui::Button::new(job);
                if ui.put(widget_rect, ok_btn).clicked() {
                    self.is_ok = Some(true);
                };

                // cancel button
                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(160.0, 0.0),
                    egui::vec2(60.0, 20.0),
                );
                let mut job = LayoutJob::default();
                job.append(
                    "Cancel",
                    0.0,
                    egui::TextFormat {
                        color: super::defines::text_color::RED,
                        ..Default::default()
                    },
                );
                let cancel_btn = egui::Button::new(job);
                if ui.put(widget_rect, cancel_btn).clicked() {
                    self.is_ok = Some(false);
                }
            });
        });
    }
}
