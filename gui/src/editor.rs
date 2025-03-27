use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui::FontFamily;
use eframe::{egui, Theme};

use egui::Visuals;
use mgit::core::repos::TomlConfig;

use crate::editor::misc::{check_git_valid, configure_text_styles, setup_custom_fonts};
use crate::editor::ops::{RepoMessage, RepoState};
use crate::editor::window::about::AboutWindow;
use crate::editor::window::dialog::{Dialog, DialogBase};
use crate::editor::window::error::ErrorWindow;
use crate::editor::window::new_branch::NewBranchWindow;
use crate::editor::window::new_tag::NewTagWindow;
use crate::editor::window::options::OptionsWindow;
use crate::toml_settings::project_settings::TomlProjectSettings;
use crate::toml_settings::user_settings::TomlUserSettings;
use crate::utils::command::CommandType;
use crate::utils::progress::OpsMessageCollector;

pub(crate) mod configuration;
pub(crate) mod layout;
pub(crate) mod menu_bar;
pub(crate) mod misc;
pub(crate) mod ops;
pub(crate) mod quick_bar;
pub(crate) mod repositories;
pub(crate) mod settings;
pub(crate) mod widgets;
pub(crate) mod window;

pub struct Editor {
    context: egui::Context,

    project_path: String,
    config_file: String,

    // recent
    toml_user_settings: TomlUserSettings,
    toml_project_settings: TomlProjectSettings,
    recent_projects: Vec<String>,

    toml_config: TomlConfig,
    remote_ref_edit_idx: i32,
    repo_states: Vec<RepoState>,

    send: Sender<RepoMessage>,
    recv: Receiver<RepoMessage>,

    // about window
    about_window: AboutWindow,
    show_about_window: bool,

    // error window
    error_window: ErrorWindow,
    show_error_window: bool,

    // option window
    options_window: OptionsWindow,
    show_options_window: bool,

    // new branch window
    new_branch_window: NewBranchWindow,
    show_new_branch_window: bool,

    // new tag window
    new_tag_window: NewTagWindow,
    show_new_tag_window: bool,

    // clean dialog
    clean_dialog: Dialog,
    show_clean_dialog: bool,

    // sync hard dialog
    sync_hard_dialog: Dialog,
    show_sync_hard_dialog: bool,

    progress: Arc<AtomicUsize>,
    ops_message_collector: OpsMessageCollector,
}

impl Default for Editor {
    fn default() -> Self {
        //let cur_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from(""));
        let (send, recv) = channel();
        let progress = Arc::new(AtomicUsize::new(0));
        Self {
            context: egui::Context::default(),

            project_path: String::new(),
            config_file: String::new(),

            toml_user_settings: TomlUserSettings::default(),
            toml_project_settings: TomlProjectSettings::default(),
            recent_projects: Vec::new(),

            toml_config: TomlConfig::default(),
            remote_ref_edit_idx: -1,
            repo_states: Vec::new(),

            send: send.clone(),
            recv,

            // about window
            about_window: Default::default(),
            show_about_window: false,

            // error window
            error_window: Default::default(),
            show_error_window: false,

            // option window
            options_window: Default::default(),
            show_options_window: false,

            // new branch window
            new_branch_window: Default::default(),
            show_new_branch_window: false,

            // new tag window
            new_tag_window: Default::default(),
            show_new_tag_window: false,

            // clean dialog
            clean_dialog: Dialog::create("Clean".to_string(), "Confirm clean?".to_string()),
            show_clean_dialog: false,

            // sync hard dialog
            sync_hard_dialog: Dialog::create(
                "Sync Hard".to_string(),
                "Confirm sync hard?".to_string(),
            ),
            show_sync_hard_dialog: false,

            progress: progress.clone(),
            ops_message_collector: OpsMessageCollector::new(send, progress),
        }
    }
}

impl Editor {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // restore app state
        #[cfg(feature = "persistence")]
        if let Some(storage) = _cc.storage {
            if let Some(state) = eframe::get_value(storage, eframe::APP_KEY) {
                slf.state = state;
            }
        }

        setup_custom_fonts(&cc.egui_ctx);
        configure_text_styles(&cc.egui_ctx);

        let mut app = Editor {
            context: cc.egui_ctx.clone(),
            ..Editor::default()
        };

        let (is_dependencies_valid, err_msg) = match check_git_valid() {
            Ok(_) => (true, String::new()),
            Err(msg) => (false, msg),
        };

        if !is_dependencies_valid {
            app.show_error_window = true;
            app.error_window = ErrorWindow::new(err_msg);
            return app;
        }

        app.load_setting();
        app.exec_ops(CommandType::Refresh);

        match app.toml_user_settings.theme {
            Some(Theme::Dark) => cc.egui_ctx.set_visuals(Visuals::dark()),
            Some(Theme::Light) => cc.egui_ctx.set_visuals(Visuals::light()),
            None => {}
        }
        app
    }
}

/// main app ui update
impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame) {
        // top view
        self.top_view(ctx);

        // content view
        self.content_view(ctx);

        // show windows
        self.handle_windows(ctx, eframe);

        // handle channel recv
        self.handle_channel_recv();
    }

    // save app state
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }
}
