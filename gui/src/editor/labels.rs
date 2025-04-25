use super::Editor;

impl Editor {
    pub(crate) fn labels_list_panel(&mut self, ui: &mut egui::Ui) {
        if let Some(repos) = &self.toml_config.repos {
            ui.heading("Labels");
            let mut labels = mgit::utils::label::collect(repos);
            labels.insert("none");
            self.toml_project_settings
                .labels
                .retain(|x| labels.contains(x.as_str()));

            let mut changed = false;
            ui.horizontal(|ui| {
                for label in labels {
                    let contains = self.toml_project_settings.labels.contains(label);
                    let mut checked = contains;
                    ui.checkbox(&mut checked, label);
                    if checked == contains {
                        continue;
                    }

                    changed = true;
                    if checked {
                        self.toml_project_settings.labels.insert(label.to_string());
                    } else {
                        self.toml_project_settings.labels.remove(label);
                    }
                }
            });

            if changed {
                self.save_project_settings();
                self.update_labels_filter();
            }
            ui.separator();
        }
    }

    pub(crate) fn update_labels_filter(&mut self) {
        let Some(repos) = &self.toml_config.repos else {
            return;
        };

        let labels = self.get_labels();
        for (repo, state) in repos.iter().zip(&mut self.repo_states) {
            state.disable_by_label = match &labels {
                Some(labels) => !mgit::utils::label::check(repo, labels),
                None => false,
            }
        }
    }
}
