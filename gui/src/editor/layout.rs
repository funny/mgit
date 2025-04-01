use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Rounding, Stroke};

use crate::editor::widgets::progress_bar::ProgressBar;
use crate::editor::Editor;
use crate::utils::defines::DEFAULT_WIDTH;

// ========================================
// ui design for app
// ========================================
impl Editor {
    /// quick bar panel of app
    pub(crate) fn top_view(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("mgit_gui_top_bar")
            .frame(
                // 该Frame拷贝自egui::TopBottomPanel的默认实现
                // 仅去掉了boarder也就是Stroke::none()
                // 使用progress bar起到separator的效果
                Frame {
                    inner_margin: Margin::symmetric(8.0, 2.0),
                    rounding: Rounding::none(),
                    fill: ctx.style().visuals.window_fill(),
                    stroke: Stroke::none(),
                    ..Default::default()
                },
            )
            .show(ctx, |ui| {
                ui.set_enabled(!self.show_error_window);
                ui.spacing_mut().item_spacing = egui::vec2(10.0, 5.0);

                // menu bar
                self.menu_bar(ui);

                // quick bar
                self.quick_bar(ui);

                ui.add_space(2.0);

                ui.add(ProgressBar::new(
                    self.progress.clone(),
                    &self.repo_states,
                    self.context.clone(),
                ));
            });
    }

    /// content_view of app
    pub(crate) fn content_view(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.set_enabled(!self.show_error_window);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.set_min_width(DEFAULT_WIDTH);
                ui.heading("Configuration");

                // configuration detail
                self.configuration_panel(ui);

                ui.separator();

                // repositories list detail
                let repos_count = match &self.toml_config.repos {
                    Some(toml_repos) => toml_repos.len(),
                    _ => 0,
                };

                self.labels_list_panel(ui);

                ui.heading(format!("Repositories ({})", repos_count));
                self.repositories_list_panel(ui);
            });
        });
    }
}
