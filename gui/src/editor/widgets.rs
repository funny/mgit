use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use eframe::egui::{vec2, NumExt, Response, Sense, Ui, Widget};
use eframe::emath::Rect;
use eframe::epaint::Stroke;

use mgit::core::repo::TomlRepo;

pub(crate) struct ProgressBar(pub(crate) f32);

impl ProgressBar {
    pub fn new(current: &Arc<AtomicUsize>, repos: &Option<Vec<TomlRepo>>) -> Self {
        let total = repos.as_ref().map(|repos| repos.len());
        let current = current.load(Ordering::Relaxed);
        let progress = if let Some(total) = total {
            if total > 0 {
                current as f32 / total as f32
            } else {
                0.0
            }
        } else {
            0.0
        };
        Self(progress)
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let progress = self.0;

        let width = ui.available_size_before_wrap().x.at_least(96.0);
        let height = 2.0f32;

        let (outer_rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        if ui.is_rect_visible(outer_rect) {
            let visual = ui.style().visuals.clone();
            let rounding = 0.0;
            let w1 = outer_rect.width();
            ui.painter().rect(
                outer_rect,
                rounding,
                visual.extreme_bg_color,
                Stroke::none(),
            );

            if progress > 0.0 {
                let inner_rect = Rect::from_min_size(
                    outer_rect.min,
                    vec2(
                        (outer_rect.width() * progress).at_least(outer_rect.height()),
                        outer_rect.height(),
                    ),
                );
                let w2 = inner_rect.width();

                println!("w1: {}, w2: {}", w1, w2);

                ui.painter()
                    .rect(inner_rect, rounding, visual.text_color(), Stroke::none());
            }
        }

        response
    }
}
