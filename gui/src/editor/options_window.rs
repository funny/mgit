use super::settings::{SyncType, TomlUserSettings};
use eframe::egui;

pub struct OptionsWindow {
    pub init_force: bool,
    pub snapshot_force: bool,
    pub snapshot_branch: bool,
    pub snapshot_ignore: String,
    pub sync_type: SyncType,
    pub sync_no_checkout: bool,
    pub sync_no_track: bool,
    pub sync_thread: u32,
    pub sync_depth: u32,
    pub fetch_thread: u32,
    pub fetch_depth: u32,
}

impl Default for OptionsWindow {
    fn default() -> Self {
        Self {
            init_force: true,

            snapshot_force: true,
            snapshot_branch: true,
            snapshot_ignore: String::new(),

            sync_type: SyncType::Stash,
            sync_no_checkout: false,
            sync_no_track: false,
            sync_thread: 4,
            sync_depth: 0,

            fetch_thread: 4,
            fetch_depth: 0,
        }
    }
}

impl OptionsWindow {
    pub fn load_option_from_settings(
        &mut self,
        toml_setting: &TomlUserSettings,
        snapshot_ignore: &Option<String>,
    ) {
        if let Some(item) = toml_setting.init_force {
            self.init_force = item;
        }
        if let Some(item) = toml_setting.snapshot_force {
            self.snapshot_force = item;
        }
        if let Some(item) = toml_setting.snapshot_branch {
            self.snapshot_branch = item;
        }
        if let Some(item) = snapshot_ignore {
            self.snapshot_ignore = item.to_owned();
        }
        if let Some(item) = &toml_setting.sync_type {
            self.sync_type = item.to_owned();
        }
        if let Some(item) = toml_setting.sync_no_checkout {
            self.sync_no_checkout = item;
        }
        if let Some(item) = toml_setting.sync_no_track {
            self.sync_no_track = item;
        }
        if let Some(item) = toml_setting.sync_thread {
            self.sync_thread = item;
        }
        if let Some(item) = toml_setting.sync_depth {
            self.sync_depth = item;
        }
        if let Some(item) = toml_setting.fetch_thread {
            self.fetch_thread = item;
        }
        if let Some(item) = toml_setting.fetch_depth {
            self.fetch_depth = item;
        }
    }
}

impl super::WindowBase for OptionsWindow {
    fn name(&self) -> String {
        format!("Command Options")
    }

    fn show(&mut self, ctx: &egui::Context, eframe: &mut eframe::Frame, open: &mut bool) {
        let width = 470.0;
        let height = 460.0;
        let screen_rect = eframe.info().window_info.size;
        let default_pos = [
            (screen_rect.x - width) * 0.5,
            (screen_rect.y - height) * 0.5,
        ];
        egui::Window::new(self.name())
            .fixed_pos(default_pos)
            .fixed_size([width, height])
            .collapsible(false)
            .open(open)
            .show(ctx, |ui| {
                use super::View;
                self.ui(ui);
            });
    }
}

impl super::View for OptionsWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);

            // init options
            egui::CollapsingHeader::new("Init")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("init_options_grid")
                        .num_columns(2)
                        .min_col_width(160.0)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("force");
                            ui.checkbox(&mut self.init_force, "");
                            ui.end_row();
                        });
                });

            // snapshot options
            egui::CollapsingHeader::new("Snapshot")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("snapshot_options_grid")
                        .num_columns(2)
                        .min_col_width(160.0)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("force");
                            ui.checkbox(&mut self.snapshot_force, "");
                            ui.end_row();

                            ui.label("branch");
                            ui.checkbox(&mut self.snapshot_branch, "");
                            ui.end_row();

                            ui.with_layout(
                                egui::Layout::top_down_justified(egui::Align::TOP),
                                |ui| {
                                    ui.label("ignore").on_hover_text("(\"\\n\" split)");
                                },
                            );
                            ui.add(
                                egui::TextEdit::multiline(&mut self.snapshot_ignore)
                                    .desired_rows(5),
                            );
                        });
                });

            // sync options
            egui::CollapsingHeader::new("Sync")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("sync_options_grid")
                        .num_columns(1)
                        .min_col_width(160.0)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("sync type");
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(10.0, 0.0);
                                ui.radio_value(&mut self.sync_type, SyncType::Normal, "normal");
                                ui.radio_value(&mut self.sync_type, SyncType::Stash, "stash");
                                ui.radio_value(&mut self.sync_type, SyncType::Hard, "hard");
                            });
                            ui.end_row();

                            ui.label("no checkout");
                            ui.checkbox(&mut self.sync_no_checkout, "");
                            ui.end_row();

                            ui.label("no track");
                            ui.checkbox(&mut self.sync_no_track, "");
                            ui.end_row();

                            ui.label("thread");
                            ui.add_sized(
                                [40.0, 20.0],
                                egui::DragValue::new(&mut self.sync_thread)
                                    .clamp_range(1..=20)
                                    .speed(1.0),
                            );
                            ui.end_row();

                            ui.label("depth");
                            ui.add_sized(
                                [40.0, 20.0],
                                egui::DragValue::new(&mut self.sync_depth)
                                    .clamp_range(0..=99999)
                                    .speed(1.0),
                            );
                        });
                });

            // fetch options
            egui::CollapsingHeader::new("Fetch")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("fetch_options_grid")
                        .num_columns(2)
                        .min_col_width(160.0)
                        .striped(false)
                        .show(ui, |ui| {
                            ui.label("thread");
                            ui.add_sized(
                                [40.0, 20.0],
                                egui::DragValue::new(&mut self.fetch_thread)
                                    .clamp_range(1..=20)
                                    .speed(1.0),
                            );
                            ui.end_row();

                            ui.label("depth");
                            ui.add_sized(
                                [40.0, 20.0],
                                egui::DragValue::new(&mut self.fetch_depth)
                                    .clamp_range(0..=99999)
                                    .speed(1.0),
                            );
                        });
                });
        });

        ui.allocate_space(ui.available_size());
    }
}
