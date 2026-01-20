use eframe::egui::{Context, Vec2};
use eframe::{egui, epaint::text::LayoutJob, Frame};

use crate::ui::style;

use super::{View, WindowBase};

pub trait DialogBase {
    fn create(name: String, content: String) -> Self;
    fn is_ok(&self) -> bool;
}

#[derive(Default)]
pub struct Dialog {
    name: String,
    content: String,
    is_ok: Option<bool>,
}

impl DialogBase for Dialog {
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

impl WindowBase for Dialog {
    fn name(&self) -> String {
        self.name.clone()
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

    fn before_show(&mut self, _: &Context, _: &mut Frame, _: &mut bool) {
        self.is_ok = None;
    }

    fn after_show(&mut self, _: &Context, _: &mut Frame, open: &mut bool) {
        if self.is_ok.is_some() {
            *open = false;
        }
    }
}

impl View for Dialog {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 20.0);
            ui.label("");
            ui.label(&self.content);

            ui.horizontal(|ui| {
                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(80.0, 0.0),
                    egui::vec2(60.0, 20.0),
                );
                let mut job = LayoutJob::default();
                job.append(
                    "Ok",
                    0.0,
                    egui::TextFormat {
                        color: style::text_color::GREEN,
                        ..Default::default()
                    },
                );
                let ok_btn = egui::Button::new(job);
                if ui.put(widget_rect, ok_btn).clicked() {
                    self.is_ok = Some(true);
                };

                let widget_rect = egui::Rect::from_min_size(
                    ui.min_rect().min + egui::vec2(160.0, 0.0),
                    egui::vec2(60.0, 20.0),
                );
                let mut job = LayoutJob::default();
                job.append(
                    "Cancel",
                    0.0,
                    egui::TextFormat {
                        color: style::text_color::RED,
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
