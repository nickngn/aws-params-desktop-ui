use crate::aws;
use crate::bridge::AsyncBridge;
use crate::state::*;
use crate::ui;

pub struct AwsParamApp {
    pub state: AppState,
    pub bridge: AsyncBridge,
    pub ssm_client: Option<aws_sdk_ssm::Client>,
    pub sm_client: Option<aws_sdk_secretsmanager::Client>,
}

impl AwsParamApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let profiles = aws::profiles::list_profiles();
        let mut state = AppState::default();
        // Auto-fill credentials from the first profile
        if let Some(first) = profiles.first() {
            state.manual_creds = aws::profiles::read_profile_creds(first);
        }
        state.available_profiles = profiles;
        Self {
            state,
            bridge: AsyncBridge::new(),
            ssm_client: None,
            sm_client: None,
        }
    }

    fn handle_result(&mut self, result: TaskResult) {
        self.state.loading = false;
        match result {
            TaskResult::ConfigLoaded(Ok(config)) => {
                self.ssm_client = Some(aws_sdk_ssm::Client::new(&config));
                self.sm_client = Some(aws_sdk_secretsmanager::Client::new(&config));
                self.state.connected = true;
                self.state.status_message = Some(("Connected".into(), StatusKind::Success));
            }
            TaskResult::ConfigLoaded(Err(e)) => {
                self.state.connected = false;
                self.state.error_dialog = Some(format!("Connection failed:\n\n{e}"));
            }
            TaskResult::ParamsList(Ok(params)) => {
                let count = params.len();
                self.state.params_list = params;
                self.state.selected_param = None;
                self.state.param_detail = None;
                self.state.status_message = Some((format!("Loaded {count} parameters"), StatusKind::Success));
            }
            TaskResult::ParamsList(Err(e)) => {
                self.state.error_dialog = Some(format!("Failed to list parameters:\n\n{e}"));
            }
            TaskResult::ParamFetched(Ok((_, val))) => {
                self.state.param_edit_buf = val.text.clone();
                self.state.param_dirty = false;
                self.state.param_detail = Some(val);
            }
            TaskResult::ParamFetched(Err(e)) => {
                self.state.error_dialog = Some(format!("Failed to get parameter:\n\n{e}"));
            }
            TaskResult::ParamCreated(Ok(msg)) | TaskResult::ParamUpdated(Ok(msg)) | TaskResult::ParamDeleted(Ok(msg)) => {
                self.state.status_message = Some((msg, StatusKind::Success));
                self.refresh_params();
            }
            TaskResult::ParamCreated(Err(e)) | TaskResult::ParamUpdated(Err(e)) | TaskResult::ParamDeleted(Err(e)) => {
                self.state.error_dialog = Some(e);
            }
            TaskResult::SecretsList(Ok(secrets)) => {
                let count = secrets.len();
                self.state.secrets_list = secrets;
                self.state.selected_secret = None;
                self.state.secret_detail = None;
                self.state.status_message = Some((format!("Loaded {count} secrets"), StatusKind::Success));
            }
            TaskResult::SecretsList(Err(e)) => {
                self.state.error_dialog = Some(format!("Failed to list secrets:\n\n{e}"));
            }
            TaskResult::SecretFetched(Ok((_, val))) => {
                match &val {
                    SecretValue::Text(t) => {
                        self.state.secret_edit_buf = t.clone();
                    }
                    SecretValue::Binary(_) => {
                        self.state.secret_edit_buf.clear();
                    }
                }
                self.state.secret_dirty = false;
                self.state.secret_detail = Some(val);
            }
            TaskResult::SecretFetched(Err(e)) => {
                self.state.error_dialog = Some(format!("Failed to get secret:\n\n{e}"));
            }
            TaskResult::SecretCreated(Ok(msg)) | TaskResult::SecretUpdated(Ok(msg)) | TaskResult::SecretDeleted(Ok(msg)) => {
                self.state.status_message = Some((msg, StatusKind::Success));
                self.refresh_secrets();
            }
            TaskResult::SecretCreated(Err(e)) | TaskResult::SecretUpdated(Err(e)) | TaskResult::SecretDeleted(Err(e)) => {
                self.state.error_dialog = Some(e);
            }
        }
    }

    fn connect(&self) {
        match self.state.auth_mode {
            AuthMode::Profile => {
                let profile = self
                    .state
                    .available_profiles
                    .get(self.state.selected_profile)
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                self.bridge.spawn(async move {
                    TaskResult::ConfigLoaded(aws::config::from_profile(&profile).await)
                });
            }
            AuthMode::ManualKeys => {
                let creds = self.state.manual_creds.clone();
                self.bridge.spawn(async move {
                    TaskResult::ConfigLoaded(aws::config::from_manual(&creds).await)
                });
            }
        }
    }

    fn refresh_params(&self) {
        if let Some(client) = &self.ssm_client {
            let client = client.clone();
            let prefix = self.state.params_filter.clone();
            self.bridge.spawn(async move {
                TaskResult::ParamsList(aws::params::list_parameters(&client, &prefix).await)
            });
        }
    }

    fn refresh_secrets(&self) {
        if let Some(client) = &self.sm_client {
            let client = client.clone();
            let prefix = self.state.secrets_filter.clone();
            self.bridge.spawn(async move {
                TaskResult::SecretsList(aws::secrets::list_secrets(&client, &prefix).await)
            });
        }
    }

    fn fetch_param_value(&self, name: &str) {
        if let Some(client) = &self.ssm_client {
            let client = client.clone();
            let name = name.to_string();
            self.bridge.spawn(async move {
                let result = aws::params::get_parameter(&client, &name).await;
                TaskResult::ParamFetched(result.map(|v| (name, v)))
            });
        }
    }

    fn fetch_secret_value(&self, name: &str) {
        if let Some(client) = &self.sm_client {
            let client = client.clone();
            let name = name.to_string();
            self.bridge.spawn(async move {
                let result = aws::secrets::get_secret_value(&client, &name).await;
                TaskResult::SecretFetched(result.map(|v| (name, v)))
            });
        }
    }
}

impl eframe::App for AwsParamApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll async results
        let results: Vec<_> = self.bridge.poll_results();
        for result in results {
            self.handle_result(result);
        }

        // Top panel: service tabs
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            let tab_action = ui::service_tabs::draw(ui, &mut self.state);
            if tab_action.changed {
                self.state.show_create_form = false;
                self.state.delete_confirm = None;
            }
        });

        // Bottom panel: status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui::common::status_bar(ui, &self.state);
        });

        // Left panel: auth
        egui::SidePanel::left("auth_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                let auth_action = ui::auth_panel::draw(ui, &mut self.state);
                if auth_action.connect {
                    self.state.loading = true;
                    self.connect();
                }
            });

        // Central panel: content
        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.state.connected {
                ui.centered_and_justified(|ui| {
                    ui.label("Connect to AWS to get started");
                });
                return;
            }

            match self.state.active_tab {
                ServiceTab::ParameterStore => {
                    let action = ui::params_view::draw(ui, &mut self.state);
                    if action.refresh {
                        self.state.loading = true;
                        self.refresh_params();
                    }
                    if let Some(name) = action.fetch_value {
                        self.state.loading = true;
                        self.fetch_param_value(&name);
                    }
                    if let Some((name, value, ptype)) = action.create {
                        self.state.loading = true;
                        if let Some(client) = &self.ssm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::ParamCreated(
                                    aws::params::create_parameter(&client, &name, &value, &ptype).await,
                                )
                            });
                        }
                    }
                    if let Some((name, value)) = action.update {
                        self.state.loading = true;
                        if let Some(client) = &self.ssm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::ParamUpdated(
                                    aws::params::update_parameter(&client, &name, &value).await,
                                )
                            });
                        }
                    }
                    if let Some(name) = action.delete {
                        self.state.loading = true;
                        if let Some(client) = &self.ssm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::ParamDeleted(
                                    aws::params::delete_parameter(&client, &name).await,
                                )
                            });
                        }
                    }
                }
                ServiceTab::SecretsManager => {
                    let action = ui::secrets_view::draw(ui, &mut self.state);
                    if action.refresh {
                        self.state.loading = true;
                        self.refresh_secrets();
                    }
                    if let Some(name) = action.fetch_value {
                        self.state.loading = true;
                        self.fetch_secret_value(&name);
                    }
                    if let Some((name, value)) = action.create {
                        self.state.loading = true;
                        if let Some(client) = &self.sm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::SecretCreated(
                                    aws::secrets::create_secret(&client, &name, &value).await,
                                )
                            });
                        }
                    }
                    if let Some((name, value)) = action.update {
                        self.state.loading = true;
                        if let Some(client) = &self.sm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::SecretUpdated(
                                    aws::secrets::update_secret(&client, &name, &value).await,
                                )
                            });
                        }
                    }
                    if let Some(name) = action.delete {
                        self.state.loading = true;
                        if let Some(client) = &self.sm_client {
                            let client = client.clone();
                            self.bridge.spawn(async move {
                                TaskResult::SecretDeleted(
                                    aws::secrets::delete_secret(&client, &name).await,
                                )
                            });
                        }
                    }
                    if let Some((name, data)) = action.download_binary {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(&name)
                            .save_file()
                        {
                            if let Err(e) = std::fs::write(&path, &data) {
                                self.state.error_dialog =
                                    Some(format!("Failed to save file:\n\n{e}"));
                            } else {
                                self.state.status_message =
                                    Some((format!("Saved to {}", path.display()), StatusKind::Success));
                            }
                        }
                    }
                }
            }
        });

        // Error dialog
        if self.state.error_dialog.is_some() {
            let mut open = true;
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(true)
                .default_width(400.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(self.state.error_dialog.as_deref().unwrap_or(""));
                    ui.add_space(8.0);
                    if ui.button("OK").clicked() {
                        self.state.error_dialog = None;
                    }
                });
            if !open {
                self.state.error_dialog = None;
            }
        }

        // Keep repainting while loading
        if self.state.loading {
            ctx.request_repaint();
        }
    }
}
