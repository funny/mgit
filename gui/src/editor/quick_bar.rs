use eframe::egui;

use crate::editor::Editor;
use crate::utils::command::CommandType;
use crate::utils::defines::hex_code;

impl Editor {
    pub(crate) fn quick_bar(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            let button_size = [96.0, 36.0];
            // fetch button
            let fetch_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("  {}\nFetch", hex_code::FETCH)),
            );
            if fetch_button_response.clicked() {
                self.exec_ops(CommandType::Fetch);
            }

            // sync button
            let sync_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!(" {}\nSync", hex_code::SYNC)),
            );
            if sync_button_response.clicked() {
                self.exec_ops(CommandType::Sync);
            }

            // sync hard button
            let sync_hard_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("     {}\nSync (Hard)", hex_code::SYNC)),
            );
            if sync_hard_button_response.clicked() {
                self.close_all_windows();
                self.show_sync_hard_dialog = true;
            }

            // refress button
            let refresh_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("   {}\nRefresh", hex_code::REFRESH)),
            );
            if refresh_button_response.clicked() {
                self.exec_ops(CommandType::Refresh);
            }
        });
    }
}
