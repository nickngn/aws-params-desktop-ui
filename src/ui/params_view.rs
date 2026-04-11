use egui::{ScrollArea, TextEdit, Ui};

use crate::state::AppState;
use crate::ui::common::{delete_confirm, DeleteAction};

pub struct ParamsAction {
    pub refresh: bool,
    pub fetch_value: Option<String>,
    pub create: Option<(String, String, String)>, // name, value, type
    pub update: Option<(String, String)>,          // name, value
    pub delete: Option<String>,
}

pub fn draw(ui: &mut Ui, state: &mut AppState) -> ParamsAction {
    let mut action = ParamsAction {
        refresh: false,
        fetch_value: None,
        create: None,
        update: None,
        delete: None,
    };

    // Toolbar
    ui.horizontal(|ui| {
        ui.label("Filter:");
        let filter_resp = ui.add(TextEdit::singleline(&mut state.params_filter).desired_width(200.0));
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
            ui.label("Create New Parameter");
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut state.new_param_name);
            });
            ui.horizontal(|ui| {
                ui.label("Type:");
                ui.radio_value(&mut state.new_param_type, "String".to_string(), "String");
                ui.radio_value(&mut state.new_param_type, "SecureString".to_string(), "SecureString");
            });
            ui.label("Value:");
            ui.add(TextEdit::multiline(&mut state.new_param_value).desired_width(f32::INFINITY).desired_rows(3));
            if ui.button("Create").clicked() && !state.new_param_name.is_empty() {
                action.create = Some((
                    state.new_param_name.clone(),
                    state.new_param_value.clone(),
                    state.new_param_type.clone(),
                ));
                state.new_param_name.clear();
                state.new_param_value.clear();
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
        .id_salt("params_list")
        .max_height(list_height)
        .show(ui, |ui| {
            if state.params_list.is_empty() && !state.loading {
                ui.label("No parameters found. Use Refresh to load.");
            }
            for (i, entry) in state.params_list.iter().enumerate() {
                let selected = state.selected_param == Some(i);
                let label = format!("{} ({})", entry.name, entry.param_type);
                if ui.selectable_label(selected, &label).clicked() && state.selected_param != Some(i) {
                    state.selected_param = Some(i);
                    state.param_dirty = false;
                    action.fetch_value = Some(entry.name.clone());
                }
            }
        });

    ui.separator();

    // Detail / edit pane
    if let Some(idx) = state.selected_param {
        if let Some(entry) = state.params_list.get(idx) {
            let name = entry.name.clone();
            let ptype = entry.param_type.clone();

            ui.horizontal(|ui| {
                ui.strong("Name:");
                ui.label(&name);
                ui.strong("Type:");
                ui.label(&ptype);
            });

            if state.param_detail.is_some() {
                ui.label("Value:");
                let resp = ui.add(
                    TextEdit::multiline(&mut state.param_edit_buf)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6),
                );
                if resp.changed() {
                    state.param_dirty = true;
                }

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(state.param_dirty, |ui| {
                        if ui.button("Save Changes").clicked() {
                            action.update = Some((name.clone(), state.param_edit_buf.clone()));
                            state.param_dirty = false;
                        }
                    });

                    // Delete
                    if let Some(ref pending) = state.delete_confirm {
                        if pending == &name {
                            match delete_confirm(ui, &name) {
                                DeleteAction::Confirm => {
                                    action.delete = Some(name);
                                    state.delete_confirm = None;
                                    state.selected_param = None;
                                    state.param_detail = None;
                                }
                                DeleteAction::Cancel => {
                                    state.delete_confirm = None;
                                }
                                DeleteAction::None => {}
                            }
                        }
                    } else if ui.button("Delete").clicked() {
                        state.delete_confirm = Some(name);
                    }
                });
            } else if state.loading {
                ui.spinner();
            }
        }
    }

    action
}
