use egui::Ui;

use crate::state::{AppState, ServiceTab};

pub struct TabAction {
    pub changed: bool,
}

pub fn draw(ui: &mut Ui, state: &mut AppState) -> TabAction {
    let prev = state.active_tab;
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.active_tab, ServiceTab::ParameterStore, "Parameter Store");
        ui.selectable_value(&mut state.active_tab, ServiceTab::SecretsManager, "Secrets Manager");
    });
    TabAction {
        changed: prev != state.active_tab,
    }
}
