use egui::{Color32, RichText, Ui};

use crate::state::{StatusKind, AppState};

pub fn status_bar(ui: &mut Ui, state: &AppState) {
    ui.horizontal(|ui| {
        if state.loading {
            ui.spinner();
            ui.label("Loading...");
        } else if let Some((msg, kind)) = &state.status_message {
            let color = match kind {
                StatusKind::Info => Color32::GRAY,
                StatusKind::Success => Color32::GREEN,
                StatusKind::Error => Color32::from_rgb(255, 80, 80),
            };
            ui.label(RichText::new(msg).color(color));
        }

        // Push "Made by NickNgn" to the right
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("Made by NickNgn").color(Color32::GRAY).small());
        });
    });
}

pub fn delete_confirm(ui: &mut Ui, name: &str) -> DeleteAction {
    let mut action = DeleteAction::None;
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Delete '{name}'?")).color(Color32::from_rgb(255, 80, 80)));
        if ui.button("Yes, Delete").clicked() {
            action = DeleteAction::Confirm;
        }
        if ui.button("Cancel").clicked() {
            action = DeleteAction::Cancel;
        }
    });
    action
}

pub enum DeleteAction {
    None,
    Confirm,
    Cancel,
}
