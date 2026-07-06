//! "Check for Updates" window — version check, download installer, install.

use std::path::PathBuf;

use eframe::egui;
use eframe::egui::Vec2;
use egui::Widget;

/// Button the user clicked — consumed by the app's event loop.
#[derive(Debug, PartialEq)]
pub(crate) enum UpgradeAction {
    Close,
    Download,
    Install,
    Retry,
}

/// State machine for the upgrade window.
/// Transitions are driven externally by `app.rs::handle_event`.
#[derive(Default)]
pub(crate) enum UpgradeState {
    #[default]
    Checking,
    UpToDate {
        current: String,
    },
    UpdateAvailable {
        current: String,
        latest: String,
        asset_url: String,
        asset_name: String,
    },
    Downloading {
        file_name: String,
    },
    ReadyToInstall {
        path: PathBuf,
    },
    Error {
        message: String,
    },
}

#[derive(Default)]
pub(crate) struct UpgradeWindow {
    pub(crate) state: UpgradeState,
    /// Set by the UI on button click; caller reads and clears it.
    pub(crate) action: Option<UpgradeAction>,
}

impl super::WindowBase for UpgradeWindow {
    fn name(&self) -> String {
        match &self.state {
            UpgradeState::UpdateAvailable { latest, .. } => {
                format!("mgit-gui v{latest} available")
            }
            _ => "Update mgit-gui".to_string(),
        }
    }

    fn width(&self) -> f32 {
        340.0
    }

    fn height(&self) -> f32 {
        match &self.state {
            UpgradeState::Checking => 140.0,
            UpgradeState::UpToDate { .. } => 140.0,
            UpgradeState::UpdateAvailable { .. } => 200.0,
            UpgradeState::Downloading { .. }  => 140.0,
            UpgradeState::ReadyToInstall { .. } => 160.0,
            UpgradeState::Error { .. } => 180.0,
        }
    }

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2] {
        [
            (screen_rect.x - self.width()) * 0.5,
            (screen_rect.y - self.height()) * 0.5,
        ]
    }
}

impl super::View for UpgradeWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Reset action each frame; buttons set it.
        self.action = None;

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 12.0);

            match &self.state {
                UpgradeState::Checking => {
                    ui.label("");
                    egui::widgets::Spinner::new().ui(ui);
                    ui.label("Checking for updates...");
                }

                UpgradeState::UpToDate { current } => {
                    ui.label("");
                    ui.label("✓ Already up to date");
                    ui.label(format!("mgit-gui v{current}"));
                    ui.label("");
                    if ui.button("OK").clicked() {
                        self.action = Some(UpgradeAction::Close);
                    }
                }

                UpgradeState::UpdateAvailable {
                    current,
                    latest,
                    ..
                } => {
                    ui.label("");
                    ui.heading("New version available!");
                    ui.label(format!("v{latest}  →  current: v{current}"));
                    ui.label("");
                    ui.horizontal(|ui| {
                        if ui.button("  Download  ").clicked() {
                            self.action = Some(UpgradeAction::Download);
                        }
                        if ui.button("  Later  ").clicked() {
                            self.action = Some(UpgradeAction::Close);
                        }
                    });
                }

                UpgradeState::Downloading { file_name } => {
                    ui.label("");
                    egui::widgets::Spinner::new().ui(ui);
                    ui.label("Downloading...");
                    ui.label(file_name.as_str());
                }

                UpgradeState::ReadyToInstall { .. } => {
                    ui.label("");
                    ui.label("✓ Download complete");
                    ui.label("Ready to install");
                    ui.label("");
                    if ui.button("  Install  ").clicked() {
                        self.action = Some(UpgradeAction::Install);
                    }
                }

                UpgradeState::Error { message } => {
                    ui.label("");
                    ui.label("✗ Update check failed");
                    ui.label(message.as_str());
                    ui.label("");
                    if ui.button("  Retry  ").clicked() {
                        self.action = Some(UpgradeAction::Retry);
                    }
                }
            }
        });
    }
}
