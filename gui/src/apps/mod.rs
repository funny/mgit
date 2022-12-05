use self::about_window::AboutWindow;
use self::defines::*;
use self::dialog::Dialog;
use self::options_window::OptionsWindow;
use self::settings::{SyncType, TomlProjectSettings, TomlUserSettings};
use eframe::egui::{self, FontFamily, FontId, TextStyle};
use mgit::commands::{TomlConfig, TomlRepo};
use std::sync::mpsc::{channel, Receiver, Sender};

pub mod defines;

mod about_window;
mod app;
mod dialog;
mod options_window;
mod settings;

#[derive(PartialEq, Clone, Copy)]
pub enum StateType {
    Disconnected,
    Updating,
    Normal,
    Warning,
    Error,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CommandType {
    None,
    Init,
    Snapshot,
    Fetch,
    Sync,
    SyncHard,
    Track,
    Clean,
    Refresh,
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

pub trait WindowBase {
    fn name(&self) -> String;

    // show window
    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool);
}

pub trait DialogBase {
    fn create(name: String, content: String) -> Self;
    fn is_ok(&self) -> bool;
}

pub trait UiExt<'t> {
    fn remote_ref_edit_button(
        &mut self,
        current_pos: impl Into<egui::Pos2>,
        idx: usize,
        branch_text: &'t mut dyn egui::TextBuffer,
        tag_text: &'t mut dyn egui::TextBuffer,
        commit_text: &'t mut dyn egui::TextBuffer,
    ) -> egui::Response;
}

pub struct App {
    project_path: String,
    config_file: String,

    // recent
    toml_user_settings: TomlUserSettings,
    toml_project_settings: TomlProjectSettings,
    recent_projects: Vec<String>,

    toml_config: TomlConfig,
    repo_states: Vec<RepoState>,

    send: Sender<(CommandType, (usize, RepoState))>,
    recv: Receiver<(CommandType, (usize, RepoState))>,

    // about window
    about_window: AboutWindow,
    about_is_open: bool,

    // option window
    options_window: OptionsWindow,
    options_is_open: bool,

    // clean dialog
    clean_dialog: Dialog,
    clean_is_open: bool,

    // sync hard dialog
    sync_hard_dialog: Dialog,
    sync_hard_is_open: bool,
}

#[derive(Clone)]
pub struct RepoState {
    current_branch: String,
    tracking_branch: String,
    track_state: StateType,
    cmp_obj: String,
    cmp_commit: String,
    cmp_changes: String,
    cmp_state: StateType,
    err_msg: String,
    no_ignore: bool,
}

impl Default for RepoState {
    fn default() -> Self {
        Self {
            current_branch: String::new(),
            tracking_branch: String::new(),
            track_state: StateType::Disconnected,
            cmp_obj: String::new(),
            cmp_commit: String::new(),
            cmp_changes: String::new(),
            cmp_state: StateType::Disconnected,
            err_msg: String::new(),
            no_ignore: true,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        //let cur_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from(""));
        let (send, recv) = channel();
        Self {
            project_path: String::new(),
            config_file: String::new(),

            toml_user_settings: TomlUserSettings::default(),
            toml_project_settings: TomlProjectSettings::default(),
            recent_projects: Vec::new(),

            toml_config: TomlConfig::default(),
            repo_states: Vec::new(),

            send,
            recv,

            // about window
            about_window: Default::default(),
            about_is_open: false,

            // option window
            options_window: Default::default(),
            options_is_open: false,

            // clean dialog
            clean_dialog: Dialog::create(format!("Clean"), format!("Confirm clean?")),
            clean_is_open: false,

            // sync hard dialog
            sync_hard_dialog: Dialog::create(format!("Sync Hard"), format!("Confirm sync hard?")),
            sync_hard_is_open: false,
        }
    }
}

pub fn setup_custom_fonts(ctx: &egui::Context) {
    // start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "nerd_font".to_owned(),
        egui::FontData::from_static(resource::NERD_FONT),
    );

    // put my font first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "nerd_font".to_owned());

    // put my font as last fallback for monospace:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "nerd_font".to_owned());

    // chinese character on window
    #[cfg(target_os = "windows")]
    {
        let font = std::fs::read("c:/Windows/Fonts/msyh.ttc").unwrap();
        fonts.font_data.insert(
            "microsoft_yahei".to_owned(),
            egui::FontData::from_owned(font),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("microsoft_yahei".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("microsoft_yahei".to_owned());
    }

    // chinese character on macos
    #[cfg(target_os = "macos")]
    {
        let font = std::fs::read("/System/Library/Fonts/PingFang.ttc").unwrap();
        fonts
            .font_data
            .insert("PingFang".to_owned(), egui::FontData::from_owned(font));

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("PingFang".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("PingFang".to_owned());
    }

    // tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::Proportional;

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(18.0, Proportional)),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(16.0, Proportional)),
        (TextStyle::Button, FontId::new(16.0, Proportional)),
        (TextStyle::Small, FontId::new(14.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

pub fn load_icon() -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let embedded_ico = resource::APP_ICON;
        let image = image::load_from_memory(embedded_ico)
            .expect("failed to open icon path")
            .into_rgba8();

        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

pub fn open_in_file_explorer(path: String) {
    if cfg!(target_os = "windows") {
        let path = path.replace("/", "\\");
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .expect("open in file explorer failed");
    } else {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .expect("open in file explorer failed");
    }
}

pub fn create_layout_job(text: String, color: egui::Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &text,
        0.0,
        egui::TextFormat {
            color,
            ..Default::default()
        },
    );
    job
}

pub fn create_truncate_layout_job(text: String, color: egui::Color32) -> egui::text::LayoutJob {
    egui::text::LayoutJob {
        sections: vec![egui::text::LayoutSection {
            leading_space: 0.0,
            byte_range: 0..text.len(),
            format: egui::TextFormat::simple(egui::FontId::default(), color),
        }],
        wrap: egui::epaint::text::TextWrapping {
            max_rows: 1,
            break_anywhere: true,
            ..Default::default()
        },
        break_on_newline: false,
        halign: egui::Align::RIGHT,
        justify: true,
        text,
        ..Default::default()
    }
}

fn cmp_toml_repo(dest: &TomlRepo, src: &TomlRepo) -> bool {
    let mut result = false;
    if dest.branch != src.branch || dest.tag != src.tag || dest.commit != src.commit {
        result = true;
    }
    result
}
