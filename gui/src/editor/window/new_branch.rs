use std::collections::HashSet;
use std::path::Path;

use eframe::egui;
use eframe::egui::Vec2;

use mgit::core::repos::TomlConfig;
use mgit::ops::NewBranchOptions;
use mgit::utils::path::PathExtension;

pub struct NewBranchWindow {
    pub root_path: String,
    pub new_branch: String,
    pub config_path: String,
    pub new_config_path: String,
    pub repos: Vec<(String, bool)>,
    pub comfirm_create: bool,
}

impl Default for NewBranchWindow {
    fn default() -> Self {
        Self {
            root_path: String::new(),
            new_branch: String::new(),
            config_path: String::new(),
            new_config_path: String::new(),
            repos: Vec::new(),
            comfirm_create: false,
        }
    }
}

impl NewBranchWindow {
    pub fn update_repo(
        &mut self,
        root_path: impl AsRef<Path>,
        config_path: impl AsRef<Path>,
        toml_config: &TomlConfig,
        ignore: &[impl AsRef<str>],
    ) {
        self.comfirm_create = false;
        self.repos.clear();
        let ignore: HashSet<String> = ignore.iter().map(|s| s.as_ref().to_string()).collect();

        self.root_path = root_path.as_ref().norm_path();
        self.config_path = config_path.as_ref().norm_path();

        if let Some(parent) = config_path.as_ref().parent() {
            self.new_config_path = parent.join("new_config.toml").norm_path();
        }

        let Some(repos) = toml_config.repos.as_ref() else {
            return;
        };

        for toml_repo in repos {
            let Some(local) = toml_repo.local.clone() else {
                continue;
            };

            let selected = !ignore.contains(&local.display_path());

            self.repos.push((local, selected));
        }
    }

    pub fn get_ignore_repos(&self) -> Vec<String> {
        self.repos
            .iter()
            .filter_map(|(repo, selected)| match selected {
                true => None,
                false => Some(repo.clone()),
            })
            .collect()
    }

    pub fn get_options(&self) -> NewBranchOptions {
        let ignore_repos: Vec<_> = self.get_ignore_repos();
        let ignore = match ignore_repos.is_empty() {
            true => None,
            false => Some(ignore_repos),
        };

        let options = NewBranchOptions::new(
            Some(self.root_path.clone()),
            Some(self.config_path.clone()),
            Some(Path::new(&self.new_config_path).to_path_buf()),
            self.new_branch.clone(),
            ignore,
        );

        options
    }
}

impl super::WindowBase for NewBranchWindow {
    fn name(&self) -> String {
        "New Branch Options".to_string()
    }

    fn width(&self) -> f32 {
        600.0
    }

    fn height(&self) -> f32 {
        600.0
    }

    fn default_pos(&self, screen_rect: &Vec2) -> [f32; 2] {
        [
            (screen_rect.x - self.width()) * 0.5,
            (screen_rect.y - self.height()) * 0.5,
        ]
    }
}

impl super::View for NewBranchWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
        ui.add_space(10.0);

        ui.label("Tips: This will create a new remote branch in force mode.");
        ui.separator();

        egui::Grid::new("new_branch_options_grid")
            .num_columns(2)
            .min_col_width(160.0)
            .striped(false)
            .show(ui, |ui| {
                ui.label("New Branch");

                let singleline_size = Vec2::new(440.0, 20.0);
                ui.add_sized(
                    singleline_size,
                    egui::TextEdit::singleline(&mut self.new_branch).hint_text("New branch name"),
                );
                ui.end_row();

                ui.label("New Config Path");
                ui.add_sized(
                    singleline_size,
                    egui::TextEdit::singleline(&mut self.new_config_path),
                );
                ui.end_row();

                ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
                    ui.label("Affected Repos");
                });

                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_min_width(430.0);
                            for (rel_path, selected) in self.repos.iter_mut() {
                                let path = Path::new(rel_path).display_path();
                                ui.checkbox(selected, path);
                            }
                        });
                    });
            });

        ui.vertical_centered(|ui| {
            ui.allocate_space(ui.available_size() - Vec2::new(0.0, 100.0));

            ui.add_enabled_ui(!self.new_branch.is_empty(), |ui| {
                let create_btn = egui::Button::new("Create");
                let response = ui.add_sized([100.0, 20.0], create_btn);

                if response.clicked() {
                    self.comfirm_create = true;
                }
            });
        });
    }
}
