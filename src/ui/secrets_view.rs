use egui::{ScrollArea, TextEdit, Ui};

use crate::state::{AppState, SecretValue};
use crate::ui::common::{delete_confirm, DeleteAction};

pub struct SecretsAction {
    pub refresh: bool,
    pub fetch_value: Option<String>,
    pub create: Option<(String, String)>, // name, value
    pub update: Option<(String, String)>, // name, value
    pub delete: Option<String>,
    pub download_binary: Option<(String, Vec<u8>)>,
}

pub fn draw(ui: &mut Ui, state: &mut AppState) -> SecretsAction {
    let mut action = SecretsAction {
        refresh: false,
        fetch_value: None,
        create: None,
        update: None,
        delete: None,
        download_binary: None,
    };

    // Toolbar
    ui.horizontal(|ui| {
        ui.label("Filter:");
        let filter_resp = ui.add(TextEdit::singleline(&mut state.secrets_filter).desired_width(200.0));
        if filter_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            action.refresh = true;
        }
        if ui.button("Refresh").clicked() {
            action.refresh = true;
        }
        if ui.button(if state.show_create_form { "Cancel New" } else { "+ New" }).clicked() {
            state.show_create_form = !state.show_create_form;
        }
    });

    ui.separator();

    // Create form (inline)
    if state.show_create_form {
        ui.group(|ui| {
            ui.label("Create New Secret");
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut state.new_secret_name);
            });
            ui.label("Value:");
            ui.add(TextEdit::multiline(&mut state.new_secret_value).desired_width(f32::INFINITY).desired_rows(3));
            if ui.button("Create").clicked() && !state.new_secret_name.is_empty() {
                action.create = Some((
                    state.new_secret_name.clone(),
                    state.new_secret_value.clone(),
                ));
                state.new_secret_name.clear();
                state.new_secret_value.clear();
                state.show_create_form = false;
            }
        });
        ui.separator();
    }

    // Split: list on top, detail on bottom
    let available = ui.available_size();
    let list_height = (available.y * 0.4).max(100.0);

    // List
    ScrollArea::vertical()
        .id_salt("secrets_list")
        .max_height(list_height)
        .show(ui, |ui| {
            if state.secrets_list.is_empty() && !state.loading {
                ui.label("No secrets found. Use Refresh to load.");
            }
            for (i, entry) in state.secrets_list.iter().enumerate() {
                let selected = state.selected_secret == Some(i);
                if ui.selectable_label(selected, &entry.name).clicked() && state.selected_secret != Some(i) {
                    state.selected_secret = Some(i);
                    state.secret_dirty = false;
                    action.fetch_value = Some(entry.name.clone());
                }
            }
        });

    ui.separator();

    // Detail / edit pane
    if let Some(idx) = state.selected_secret {
        if let Some(entry) = state.secrets_list.get(idx) {
            let name = entry.name.clone();

            ui.horizontal(|ui| {
                ui.strong("Name:");
                ui.label(&name);
                if let Some(desc) = &entry.description {
                    ui.strong("Desc:");
                    ui.label(desc);
                }
            });

            if let Some(detail) = &state.secret_detail {
                match detail {
                    SecretValue::Text(_) => {
                        ui.label("Value:");
                        let resp = ui.add(
                            TextEdit::multiline(&mut state.secret_edit_buf)
                                .desired_width(f32::INFINITY)
                                .desired_rows(6),
                        );
                        if resp.changed() {
                            state.secret_dirty = true;
                        }

                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(state.secret_dirty, |ui| {
                                if ui.button("Save Changes").clicked() {
                                    action.update = Some((name.clone(), state.secret_edit_buf.clone()));
                                    state.secret_dirty = false;
                                }
                            });

                            draw_delete(ui, state, &name, &mut action);
                        });
                    }
                    SecretValue::Binary(data) => {
                        ui.label(format!("Binary value ({} bytes)", data.len()));
                        if ui.button("Download").clicked() {
                            action.download_binary = Some((name.clone(), data.clone()));
                        }
                        ui.horizontal(|ui| {
                            draw_delete(ui, state, &name, &mut action);
                        });
                    }
                }
            } else if state.loading {
                ui.spinner();
            }
        }
    }

    action
}

fn draw_delete(ui: &mut egui::Ui, state: &mut AppState, name: &str, action: &mut SecretsAction) {
    if let Some(ref pending) = state.delete_confirm {
        if pending == name {
            match delete_confirm(ui, name) {
                DeleteAction::Confirm => {
                    action.delete = Some(name.to_string());
                    state.delete_confirm = None;
                    state.selected_secret = None;
                    state.secret_detail = None;
                }
                DeleteAction::Cancel => {
                    state.delete_confirm = None;
                }
                DeleteAction::None => {}
            }
        }
    } else if ui.button("Delete").clicked() {
        state.delete_confirm = Some(name.to_string());
    }
}
