use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use eframe::egui;
use eframe::egui::{vec2, NumExt, Rect, Response, Sense, Stroke, Ui, Widget};

use crate::app::context::RepoState;

pub(crate) struct ProgressBar {
    pub(crate) progress: f32,
    current: Arc<AtomicUsize>,
    context: egui::Context,
}

impl ProgressBar {
    pub fn new(current: Arc<AtomicUsize>, repos: &[RepoState], context: egui::Context) -> Self {
        let total = repos
            .iter()
            .filter(|repo| repo.no_ignore && !repo.disable_by_label)
            .count();

        let current_rate = current.load(Ordering::Relaxed);
        let progress = if total > 0 {
            current_rate as f32 / total as f32
        } else {
            0.0
        };
        Self {
            progress,
            current,
            context,
        }
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let progress = self.progress;

        let width = ui.available_size_before_wrap().x.at_least(96.0);
        let height = 2.0f32;

        let (outer_rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());

        if ui.is_rect_visible(outer_rect) {
            let visual = ui.style().visuals.clone();
            let rounding = 0.0;

            ui.painter().hline(
                outer_rect.x_range(),
                outer_rect.center().y,
                visual.widgets.noninteractive.bg_stroke,
            );

            let inner_rect = Rect::from_min_size(
                outer_rect.min,
                vec2(
                    (outer_rect.width() * progress).at_least(0.0),
                    outer_rect.height(),
                ),
            );

            ui.painter().rect(
                inner_rect,
                rounding,
                egui::Color32::WHITE,
                Stroke::new(0.0, visual.text_color()),
                egui::StrokeKind::Inside,
            );

            if progress >= 1.0 {
                let current = self.current;
                let context = self.context.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(1));
                    current.store(0, Ordering::Relaxed);
                    context.request_repaint();
                });
            }
        }

        response
    }
}
