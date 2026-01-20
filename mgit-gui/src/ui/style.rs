use eframe::egui;
use eframe::egui::{Color32, FontId, TextStyle};
use egui::IconData;
use mgit::Colour;

pub const DEFAULT_WIDTH: f32 = 1160.0;
pub const DEFAULT_HEIGHT: f32 = 740.0;

pub const MENU_BOX_WIDTH: f32 = 200.0;

#[allow(dead_code)]
pub mod resource {
    pub const NERD_FONT: &[u8] =
        include_bytes!("../../resource/Fira Code Medium Nerd Font Complete.ttf");

    pub const APP_ICON: &[u8] = include_bytes!("../../resource/logo64x64.ico");

    pub const LOGO: &[u8] = include_bytes!("../../resource/logo128x128.png");
}

#[allow(dead_code)]
pub mod text_color {
    use super::*;

    pub const BLUE: Color32 = Color32::from_rgb(87, 167, 250);
    pub const RED: Color32 = Color32::from_rgb(202, 86, 44);
    pub const GREEN: Color32 = Color32::from_rgb(0, 200, 0);
    pub const YELLOW: Color32 = Color32::from_rgb(194, 169, 19);
    pub const PURPLE: Color32 = Color32::from_rgb(227, 103, 229);
    pub const DARK_PURPLE: Color32 = Color32::from_rgb(140, 60, 140);
    pub const GRAY: Color32 = Color32::GRAY;
    pub const LIGHT_GRAY: Color32 = Color32::from_rgba_premultiplied(50, 50, 50, 50);
    pub const BLACK: Color32 = Color32::BLACK;
    pub const WHITE: Color32 = Color32::WHITE;

    pub fn from(color: &Option<Colour>) -> Color32 {
        if let Some(color) = color {
            match color {
                Colour::Black => BLACK,
                Colour::Red => RED,
                Colour::Green => GREEN,
                Colour::Yellow => YELLOW,
                Colour::Blue => BLUE,
                Colour::Purple => PURPLE,
                Colour::White => WHITE,
                _ => LIGHT_GRAY,
            }
        } else {
            WHITE
        }
    }
}

#[allow(unused)]
pub mod console_option {
    pub const CREATE_NEW_CONSOLE: u32 = 0x00000010;
    pub const DETACHED_PROCESS: u32 = 0x00000008;
    pub const CREATE_NO_WINDOW: u32 = 0x08000000;
}

#[allow(unused)]
pub mod hex_code {
    pub const DISCONNECTED: &str = "\u{ea71}";
    pub const UPDATING: &str = "\u{f021}";
    pub const NORMAL: &str = "\u{ea71}";
    pub const WARNING: &str = "\u{ea6c}";
    pub const ERROR: &str = "\u{ea87}";
    pub const ISSUE: &str = "\u{f41b}";

    pub const ARROW_RIGHT_BOLD: &str = "\u{fc32}";
    pub const ARROW_RIGHT_BOX: &str = "\u{fbc0}";
    pub const ARROW_DOWN: &str = "\u{f544}";
    pub const FOLDER: &str = "\u{ea83}";
    pub const LINK_EXTERNAL: &str = "\u{f465}";
    pub const FILE: &str = "\u{ea7b}";
    pub const EDIT: &str = "\u{f044}";
    pub const GIT: &str = "\u{eb00}";

    pub const INIT: &str = "\u{eba0}";
    pub const SNAPSHOT: &str = "\u{f030}";
    pub const FETCH: &str = "\u{f409}";
    pub const SYNC: &str = "\u{fb3e}";
    pub const TRACK: &str = "\u{f73e}";
    pub const CLEAN: &str = "\u{eabf}";
    pub const REFRESH: &str = "\u{f94f}";

    pub const REPOSITORY: &str = "\u{ea62}";
    pub const URL: &str = "\u{f838}";
    pub const BRANCH: &str = "\u{fb2b}";
    pub const TAG: &str = "\u{f9f8}";
    pub const COMMIT: &str = "\u{fc16}";
    pub const LABEL: &str = "\u{f815}";
    pub const HISTORY: &str = "\u{ea42}";
    pub const DROPDOWN: &str = "\u{25bc}";
}

pub fn setup_custom_fonts(ctx: &egui::Context) {
    // start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();

    // .ttf and .otf files supported.
    fonts.font_data.insert(
        "nerd_font".to_owned(),
        egui::FontData::from_static(resource::NERD_FONT).into(),
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
            egui::FontData::from_owned(font).into(),
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
        let font_pair = [
            ("/System/Library/Fonts/PingFang.ttc", "PingFang"),
            ("/System/Library/Fonts/STHeiti Light.ttc", "STHeiti Light"),
        ];

        let mut load_font = false;
        for (path, key) in font_pair {
            let Ok(font) = std::fs::read(path) else {
                continue;
            };
            fonts
                .font_data
                .insert(key.to_owned(), egui::FontData::from_owned(font).into());

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(key.to_owned());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(key.to_owned());
            load_font = true;
            break;
        }

        if !load_font {
            panic!("please install PingFang or STHeiti font")
        }
    }

    // tell egui to use these fonts:
    ctx.set_fonts(fonts);
}

pub fn configure_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;

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

pub fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(resource::LOGO)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
