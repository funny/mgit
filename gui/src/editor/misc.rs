use eframe::egui;
use eframe::egui::{FontId, TextStyle};
use home::home_dir;

use crate::utils::defines::{resource, GIT_VERSION};

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
    use super::FontFamily::Proportional;

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

pub fn open_in_file_explorer(path: &str) {
    if cfg!(target_os = "windows") {
        let path = path.replace('/', "\\");
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    } else {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .expect("open in file explorer failed");
    }
}

pub fn open_repo_in_fork(repo_path: &str) {
    if cfg!(target_os = "windows") {
        let fork = format!(
            "{}/AppData/Local/Fork/Fork.exe",
            home_dir().unwrap().display()
        );
        let _ = std::process::Command::new(fork).arg(repo_path).spawn();
    } else {
        let _ = std::process::Command::new("open")
            .arg("-a")
            .arg("Fork")
            .arg(repo_path)
            .spawn();
    }
}

pub fn check_git_valid() -> Result<(), String> {
    // make sure git is installed
    #[cfg(target_os = "windows")]
    let output = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        std::process::Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .arg("/C")
            .arg("git --version")
            .output()
            .expect("command failed to start")
    };

    #[cfg(not(target_os = "windows"))]
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .expect("command failed to start");

    if !output.status.success() {
        return Err(String::from("git is not found!\n"));
    }

    // make sure git version = GIT_VERSION
    let version_desc = String::from_utf8(output.stdout).expect("mgit error");
    let re = regex::Regex::new(r"(?P<version>(\d+\.\d+\.\d+))").unwrap();
    if let Some(caps) = re.captures(&version_desc) {
        let version = caps["version"].to_string();
        let expect_version = semver::VersionReq::parse(GIT_VERSION).expect("semver error");
        let current_version = semver::Version::parse(&version).expect("semver error");

        match expect_version.matches(&current_version) {
            true => Ok(()),
            false => Err(format!(
                "git version {} is required, current version is {}\n",
                GIT_VERSION, version
            )),
        }
    } else {
        Err(String::from("failed to get git version\n"))
    }
}
