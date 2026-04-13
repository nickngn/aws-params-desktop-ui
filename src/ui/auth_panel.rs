use egui::Ui;

use crate::aws;
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

    // Profile selector — shown in both modes
    ui.label("Profile:");
    let prev_profile = state.selected_profile;
    egui::ComboBox::from_id_salt("profile_select")
        .width(160.0)
        .show_index(ui, &mut state.selected_profile, state.available_profiles.len(), |i| {
            state.available_profiles.get(i).cloned().unwrap_or_default()
        });

    // Auto-fill manual creds when profile selection changes
    if state.selected_profile != prev_profile {
        if let Some(profile_name) = state.available_profiles.get(state.selected_profile) {
            state.manual_creds = aws::profiles::read_profile_creds(profile_name);
        }
    }

    // Show credentials fields in Manual mode (editable) or Profile mode (read-only)
    match state.auth_mode {
        AuthMode::Profile => {
            if !state.manual_creds.access_key.is_empty() {
                ui.add_space(4.0);
                ui.label("Loaded from profile:");
                ui.horizontal(|ui| {
                    ui.label("Access Key:");
                    ui.label(&mask_key(&state.manual_creds.access_key));
                });
                ui.horizontal(|ui| {
                    ui.label("Region:");
                    ui.label(if state.manual_creds.region.is_empty() {
                        "(default)"
                    } else {
                        &state.manual_creds.region
                    });
                });
            }
        }
        AuthMode::ManualKeys => {
            ui.add_space(4.0);
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

/// Mask a key like "AKIA1234ABCD5678EFGH" → "AKIA...EFGH"
fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}
