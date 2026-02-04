use eframe::egui;

use crate::app::events::CommandType;
use crate::app::events::OpsCommand;

use super::{
    AboutWindow, Dialog, DialogBase, ErrorWindow, NewBranchWindow, NewTagWindow, OptionsWindow,
    WindowBase,
};

#[derive(Default)]
pub(crate) struct WindowManagerOutput {
    pub(crate) commands: Vec<CommandType>,
    pub(crate) ops_commands: Vec<OpsCommand>,
    pub(crate) save_options: bool,
    pub(crate) save_snapshot_ignore: bool,
    pub(crate) save_new_branch_option: bool,
    pub(crate) save_new_tag_option: bool,
    pub(crate) retry_config_save: bool,
    pub(crate) exit_app: bool,
}

pub(crate) struct WindowManager {
    pub(crate) about: AboutWindow,
    pub(crate) about_open: bool,

    pub(crate) error: ErrorWindow,
    pub(crate) error_open: bool,
    pub(crate) error_exit_app: bool,

    pub(crate) options: OptionsWindow,
    pub(crate) options_open: bool,

    pub(crate) new_branch: NewBranchWindow,
    pub(crate) new_branch_open: bool,

    pub(crate) new_tag: NewTagWindow,
    pub(crate) new_tag_open: bool,

    pub(crate) clean_dialog: Dialog,
    pub(crate) clean_dialog_open: bool,

    pub(crate) sync_hard_dialog: Dialog,
    pub(crate) sync_hard_dialog_open: bool,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            about: AboutWindow::default(),
            about_open: false,
            error: ErrorWindow::default(),
            error_open: false,
            error_exit_app: false,
            options: OptionsWindow::default(),
            options_open: false,
            new_branch: NewBranchWindow::default(),
            new_branch_open: false,
            new_tag: NewTagWindow::default(),
            new_tag_open: false,
            clean_dialog: Dialog::create("Clean".to_string(), "Confirm clean?".to_string()),
            clean_dialog_open: false,
            sync_hard_dialog: Dialog::create(
                "Sync Hard".to_string(),
                "Confirm sync hard?\n\nThis will force delete:\n.git/shallow.lock\n.git/index.lock".to_string(),
            ),
            sync_hard_dialog_open: false,
        }
    }
}

impl WindowManager {
    pub(crate) fn is_error_open(&self) -> bool {
        self.error_open
    }

    pub(crate) fn close_all(&mut self) {
        self.about_open = false;
        self.options_open = false;
        self.clean_dialog_open = false;
        self.sync_hard_dialog_open = false;
        self.new_branch_open = false;
        self.new_tag_open = false;
    }

    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        eframe: &mut eframe::Frame,
    ) -> WindowManagerOutput {
        let mut out = WindowManagerOutput::default();

        self.about.show(ctx, eframe, &mut self.about_open);

        let options_open_before = self.options_open;
        self.options.show(ctx, eframe, &mut self.options_open);
        if options_open_before && !self.options_open {
            out.save_options = true;
            out.save_snapshot_ignore = true;
        }

        if self.error_open {
            self.error.show(ctx, eframe, &mut self.error_open);
            if self.error.take_retry_requested() {
                out.retry_config_save = true;
            }
            if !self.error_open {
                if self.error_exit_app {
                    out.exit_app = true;
                }
                self.error_exit_app = false;
            }
        }

        self.clean_dialog
            .show(ctx, eframe, &mut self.clean_dialog_open);
        if self.clean_dialog.is_ok() {
            out.commands.push(CommandType::Clean);
        }

        self.sync_hard_dialog
            .show(ctx, eframe, &mut self.sync_hard_dialog_open);
        if self.sync_hard_dialog.is_ok() {
            out.commands.push(CommandType::SyncHard);
        }

        let new_branch_open_before = self.new_branch_open;
        self.new_branch.show(ctx, eframe, &mut self.new_branch_open);
        if new_branch_open_before && !self.new_branch_open {
            out.save_new_branch_option = true;
        }
        if self.new_branch.confirm_create {
            self.new_branch.confirm_create = false;
            self.new_branch_open = false;
            out.save_new_branch_option = true;
            out.ops_commands
                .push(OpsCommand::CreateBranch(self.new_branch.get_options()));
        }

        let new_tag_open_before = self.new_tag_open;
        self.new_tag.show(ctx, eframe, &mut self.new_tag_open);
        if new_tag_open_before && !self.new_tag_open {
            out.save_new_tag_option = true;
        }
        if self.new_tag.confirm_create {
            self.new_tag.confirm_create = false;
            self.new_tag_open = false;
            out.save_new_tag_option = true;
            out.ops_commands
                .push(OpsCommand::CreateTag(self.new_tag.get_options()));
        }

        out
    }
}
