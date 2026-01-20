use eframe::egui;
use tracing::info;

use crate::app::events::{Action, Event};
use crate::app::GuiApp;

impl GuiApp {
    pub(crate) fn handle_windows(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        let out = self.windows.show(ctx, eframe);

        if out.save_options {
            info!("ui_window_options_closed_save");
            self.enqueue_event(Event::Action(Action::SaveOptions));
        }

        if out.save_snapshot_ignore {
            info!("ui_window_options_closed_save_snapshot_ignore");
            self.enqueue_event(Event::Action(Action::SaveSnapshotIgnore));
        }

        if out.save_new_branch_option {
            info!("ui_window_new_branch_closed_save");
            self.enqueue_event(Event::Action(Action::SaveNewBranchOption));
        }

        if out.save_new_tag_option {
            info!("ui_window_new_tag_closed_save");
            self.enqueue_event(Event::Action(Action::SaveNewTagOption));
        }

        if out.retry_config_save {
            info!("ui_window_error_retry_config_save");
            self.enqueue_event(Event::Action(Action::RetryConfigSave));
        }

        let mut ops_commands: Vec<_> = out.commands.into_iter().map(Into::into).collect();
        ops_commands.extend(out.ops_commands);
        if !ops_commands.is_empty() {
            info!(
                command_count = ops_commands.len(),
                "ui_window_confirm_ops_batch"
            );
            self.enqueue_event(Event::Action(Action::RunOpsBatch(ops_commands)));
        }

        if out.exit_app {
            info!("ui_window_exit_app");
            self.enqueue_event(Event::Action(Action::ExitApp));
        }
    }

    pub(crate) fn close_all_windows(&mut self) {
        self.windows.close_all();
    }
}
