use eframe::egui::Color32;

pub const DEFAULT_WIDTH: f32 = 1160.0;
pub const DEFAULT_HEIGHT: f32 = 740.0;

pub const MENU_BOX_WIDTH: f32 = 200.0;

pub const MGIT_DIR: &str = "mgit";
pub const MGIT_VERSION: &str = "~1.1.3";

pub mod resource {
    pub const NERD_FONT: &'static [u8] =
        include_bytes!("../../resource/Fira Code Medium Nerd Font Complete.ttf");

    pub const APP_ICON: &'static [u8] = include_bytes!("../../resource/logo64x64.ico");

    pub const LOGO: &'static [u8] = include_bytes!("../../resource/logo128x128.png");
}

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
}
