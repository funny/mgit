use std::collections::HashSet;
use std::path::Path;

use eframe::egui;
use eframe::egui::Vec2;

use mgit::config::MgitConfig;
use mgit::ops::NewTagOptions;
use mgit::utils::path::PathExtension;

use super::repo_selection::RepoSelection;

pub struct NewTagWindow {
    pub root_path: String,
    pub new_tag: String,
    pub config_path: String,
    pub repos: Vec<RepoSelection>,
    pub push: bool,
    pub confirm_create: bool,
}

impl Default for NewTagWindow {
    fn default() -> Self {
        Self {
            root_path: String::new(),
            new_tag: String::new(),
            config_path: String::new(),
            repos: Vec::new(),
            push: true,
            confirm_create: false,
        }
    }
}

impl NewTagWindow {
    pub fn update_settings(
        &mut self,
        root_path: impl AsRef<Path>,
        config_path: impl AsRef<Path>,
        mgit_config: &MgitConfig,
        new_tag: impl ToString,
        push: bool,
        ignore: &[impl AsRef<str>],
    ) {
        self.confirm_create = false;
        self.repos.clear();
        let ignore: HashSet<String> = ignore.iter().map(|s| s.as_ref().to_string()).collect();

        self.root_path = root_path.as_ref().norm_path();
        self.config_path = config_path.as_ref().norm_path();
        self.new_tag = new_tag.to_string();
        self.push = push;

        let Some(repos) = mgit_config.repos.as_ref() else {
            return;
        };

        for repo_config in repos {
            let Some(local) = repo_config.local.clone() else {
                continue;
            };

            let selected = !ignore.contains(&local.display_path());

            self.repos.push(RepoSelection {
                repo: local,
                selected,
            });
        }
    }

    pub fn get_ignore_repos(&self) -> Vec<String> {
        self.repos
            .iter()
            .filter_map(|repo| match repo.selected {
                true => None,
                false => Some(repo.repo.clone()),
            })
            .collect()
    }

    pub fn get_options(&self) -> NewTagOptions {
        let ignore_repos: Vec<_> = self.get_ignore_repos();
        let ignore = match ignore_repos.is_empty() {
            true => None,
            false => Some(ignore_repos),
        };

        NewTagOptions::new(
            Some(self.root_path.clone()),
            Some(self.config_path.clone()),
            self.new_tag.clone(),
            self.push,
            ignore,
        )
    }
}

impl super::WindowBase for NewTagWindow {
    fn name(&self) -> String {
        "New Tag Options".to_string()
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

impl super::View for NewTagWindow {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 10.0);
        ui.add_space(10.0);

        ui.label("Tips: This will create a new tag in force mode.");
        ui.separator();

        egui::Grid::new("new_tag_options_grid")
            .num_columns(2)
            .min_col_width(160.0)
            .striped(false)
            .show(ui, |ui| {
                ui.label("New Tag");

                let singleline_size = Vec2::new(440.0, 20.0);
                ui.add_sized(
                    singleline_size,
                    egui::TextEdit::singleline(&mut self.new_tag).hint_text("New branch name"),
                );
                ui.end_row();

                ui.label("Push");
                ui.checkbox(&mut self.push, "");
                ui.end_row();

                ui.with_layout(egui::Layout::top_down_justified(egui::Align::TOP), |ui| {
                    ui.label("Affected Repos");
                });

                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_min_width(430.0);
                            for repo in self.repos.iter_mut() {
                                let path = Path::new(&repo.repo).display_path();
                                ui.checkbox(&mut repo.selected, path);
                            }
                        });
                    });
            });

        ui.vertical_centered(|ui| {
            ui.allocate_space(ui.available_size() - Vec2::new(0.0, 100.0));

            ui.add_enabled_ui(!self.new_tag.is_empty(), |ui| {
                let create_btn = egui::Button::new("Create");
                let response = ui.add_sized([100.0, 20.0], create_btn);

                if response.clicked() {
                    self.confirm_create = true;
                }
            });
        });
    }
}
