use std::sync::mpsc::Sender;

use eframe::egui;
use tracing::info;

use crate::app::events::CommandType;
use crate::app::events::{Action, Event};
use crate::ui::style::hex_code;

pub struct QuickBarPanel;

impl QuickBarPanel {
    pub(crate) fn show(
        ui: &mut egui::Ui,
        event_tx: &Sender<Event>,
        sync_hard_dialog_open: &mut bool,
        close_windows: impl FnOnce(),
    ) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            let button_size = [96.0, 36.0];
            let fetch_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("  {}\nFetch", hex_code::FETCH)),
            );
            if fetch_button_response.clicked() {
                info!("ui_click_quickbar_fetch");
                let _ = event_tx.send(Event::Action(Action::RunOps(CommandType::Fetch.into())));
            }

            let sync_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!(" {}\nSync", hex_code::SYNC)),
            );
            if sync_button_response.clicked() {
                info!("ui_click_quickbar_sync");
                let _ = event_tx.send(Event::Action(Action::RunOps(CommandType::Sync.into())));
            }

            let sync_hard_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("     {}\nSync (Hard)", hex_code::SYNC)),
            );
            if sync_hard_button_response.clicked() {
                info!("ui_click_quickbar_sync_hard_open_dialog");
                close_windows();
                *sync_hard_dialog_open = true;
            }

            let refresh_button_response = ui.add_sized(
                button_size,
                egui::Button::new(format!("   {}\nRefresh", hex_code::REFRESH)),
            );
            if refresh_button_response.clicked() {
                info!("ui_click_quickbar_refresh");
                let _ = event_tx.send(Event::Action(Action::Refresh));
            }
        });
    }
}
