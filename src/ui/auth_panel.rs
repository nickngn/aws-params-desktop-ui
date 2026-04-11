use egui::Ui;

use crate::state::{AppState, AuthMode};

pub struct AuthAction {
    pub connect: bool,
}

pub fn draw(ui: &mut Ui, state: &mut AppState) -> AuthAction {
    let mut action = AuthAction { connect: false };

    ui.heading("AWS Auth");
    ui.separator();

    ui.radio_value(&mut state.auth_mode, AuthMode::Profile, "AWS Profile");
    ui.radio_value(&mut state.auth_mode, AuthMode::ManualKeys, "Manual Keys");

    ui.add_space(8.0);

    match state.auth_mode {
        AuthMode::Profile => {
            ui.label("Profile:");
            egui::ComboBox::from_id_salt("profile_select")
                .width(160.0)
                .show_index(ui, &mut state.selected_profile, state.available_profiles.len(), |i| {
                    state.available_profiles.get(i).cloned().unwrap_or_default()
                });
        }
        AuthMode::ManualKeys => {
            ui.label("Access Key:");
            ui.text_edit_singleline(&mut state.manual_creds.access_key);

            ui.label("Secret Key:");
            ui.add(egui::TextEdit::singleline(&mut state.manual_creds.secret_key).password(true));

            ui.label("Session Token (optional):");
            ui.add(egui::TextEdit::singleline(&mut state.manual_creds.session_token).password(true));

            ui.label("Region:");
            ui.text_edit_singleline(&mut state.manual_creds.region);
        }
    }

    ui.add_space(8.0);

    let connect_label = if state.connected { "Reconnect" } else { "Connect" };
    if ui.button(connect_label).clicked() {
        action.connect = true;
    }

    if state.connected {
        ui.colored_label(egui::Color32::GREEN, "Connected");
    }

    action
}
