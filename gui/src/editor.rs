use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use eframe::egui;
use eframe::egui::FontFamily;

use mgit::core::repos::TomlConfig;

use crate::editor::misc::{check_git_valid, configure_text_styles, setup_custom_fonts};
use crate::editor::ops::{RepoMessage, RepoState};
use crate::editor::window::about::AboutWindow;
use crate::editor::window::dialog::{Dialog, DialogBase};
use crate::editor::window::error::ErrorWindow;
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
    about_is_open: bool,

    // error window
    error_window: ErrorWindow,
    error_is_open: bool,

    // option window
    options_window: OptionsWindow,
    options_is_open: bool,

    // clean dialog
    clean_dialog: Dialog,
    clean_is_open: bool,

    // sync hard dialog
    sync_hard_dialog: Dialog,
    sync_hard_is_open: bool,

    progress: Arc<AtomicUsize>,
    ops_message_collector: OpsMessageCollector,
}

impl Default for Editor {
    fn default() -> Self {
        //let cur_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from(""));
        let (send, recv) = channel();
        let progress = Arc::new(AtomicUsize::new(0));
        Self {
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
            about_is_open: false,

            // error window
            error_window: Default::default(),
            error_is_open: false,

            // option window
            options_window: Default::default(),
            options_is_open: false,

            // clean dialog
            clean_dialog: Dialog::create("Clean".to_string(), "Confirm clean?".to_string()),
            clean_is_open: false,

            // sync hard dialog
            sync_hard_dialog: Dialog::create(
                "Sync Hard".to_string(),
                "Confirm sync hard?".to_string(),
            ),
            sync_hard_is_open: false,

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

        let mut app = Editor::default();

        let (is_dependencies_valid, err_msg) = match check_git_valid() {
            Ok(_) => (true, String::new()),
            Err(msg) => (false, msg),
        };

        if !is_dependencies_valid {
            app.error_is_open = true;
            app.error_window = ErrorWindow::new(err_msg);
            return app;
        }

        app.load_setting();
        app.exec_ops(CommandType::Refresh);
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
