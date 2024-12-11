use eframe::egui;
use eframe::egui::{Ui, Vec2};

use crate::editor::window::dialog::DialogBase;
use crate::editor::Editor;
use crate::utils::command::CommandType;

pub(crate) mod about;
pub(crate) mod dialog;
pub(crate) mod error;
pub(crate) mod new_branch;
pub(crate) mod new_tag;
pub(crate) mod options;

impl Editor {
    /// part of app
    pub(crate) fn handle_windows(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        // show about window
        self.about_window
            .show(ctx, eframe, &mut self.show_about_window);

        // show options window
        self.options_window
            .show(ctx, eframe, &mut self.show_options_window);

        if self.show_options_window {
            self.toml_user_settings.save_options(&self.options_window);
            self.save_snapshot_ignore();
        }

        // show error window
        if self.show_error_window {
            self.error_window
                .show(ctx, eframe, &mut self.show_error_window);
            if !self.show_error_window {
                std::process::exit(0x0100);
            }
        }

        // show clean dialog
        self.clean_dialog
            .show(ctx, eframe, &mut self.show_clean_dialog);

        if self.clean_dialog.is_ok() {
            self.exec_ops(CommandType::Clean);
        }

        // show sync hard dialog
        self.sync_hard_dialog
            .show(ctx, eframe, &mut self.show_sync_hard_dialog);

        if self.sync_hard_dialog.is_ok() {
            self.exec_ops(CommandType::SyncHard);
        }

        // show new branch window
        self.new_branch_window
            .show(ctx, eframe, &mut self.show_new_branch_window);

        if self.show_new_branch_window {
            self.save_new_branch_option();
        }

        if self.new_branch_window.comfirm_create {
            self.new_branch_window.comfirm_create = false;
            self.show_new_branch_window = false;
            self.exec_ops(CommandType::NewBranch)
        }

        // show new tag window
        self.new_tag_window
            .show(ctx, eframe, &mut self.show_new_tag_window);

        if self.show_new_tag_window {
            self.save_new_tag_option();
        }

        if self.new_tag_window.comfirm_create {
            self.new_tag_window.comfirm_create = false;
            self.show_new_tag_window = false;
            self.exec_ops(CommandType::NewTag)
        }
    }

    pub(crate) fn close_all_windows(&mut self) {
        self.show_about_window = false;
        self.show_options_window = false;
        self.show_clean_dialog = false;
        self.show_sync_hard_dialog = false;
        self.show_new_branch_window = false;
    }
}

pub trait View {
    fn ui(&mut self, ui: &mut Ui);
}

pub trait WindowBase: View {
    fn name(&self) -> String;

    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2];

    #[allow(unused_variables)]
    fn before_show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {}

    // show window
    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {
        let default_pos = self.default_pos(&eframe.info().window_info.size);

        self.before_show(ctx, eframe, open);
        egui::Window::new(self.name())
            .fixed_pos(default_pos)
            .fixed_size([self.width(), self.height()])
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        self.after_show(ctx, eframe, open);
    }

    #[allow(unused_variables)]
    fn after_show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {}
}
