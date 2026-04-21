#[derive(PartialEq, Clone)]
pub enum AuthMode {
    Profile,
    ManualKeys,
}

#[derive(Default, Clone)]
pub struct ManualCreds {
    pub access_key: String,
    pub secret_key: String,
    pub session_token: String,
    pub region: String,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ServiceTab {
    SecretsManager,
    ParameterStore,
}

#[derive(Clone)]
pub struct SecretEntry {
    pub name: String,
    pub arn: String,
    pub description: Option<String>,
}

#[derive(Clone)]
pub enum SecretValue {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Clone)]
pub struct ParamEntry {
    pub name: String,
    pub param_type: String,
    pub version: i64,
}

#[derive(Clone)]
pub struct ParamValue {
    pub text: String,
    pub param_type: String,
}

pub enum TaskResult {
    ConfigLoaded(Result<aws_types::SdkConfig, String>),
    SecretsList(Result<Vec<SecretEntry>, String>),
    SecretFetched(Result<(String, SecretValue), String>),
    SecretCreated(Result<String, String>),
    SecretUpdated(Result<String, String>),
    SecretDeleted(Result<String, String>),
    ParamsList(Result<Vec<ParamEntry>, String>),
    ParamFetched(Result<(String, ParamValue), String>),
    ParamCreated(Result<String, String>),
    ParamUpdated(Result<String, String>),
    ParamDeleted(Result<String, String>),
}

#[derive(PartialEq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

pub struct AppState {
    // Auth
    pub auth_mode: AuthMode,
    pub available_profiles: Vec<String>,
    pub selected_profile: usize,
    pub manual_creds: ManualCreds,
    pub connected: bool,

    // Service tab
    pub active_tab: ServiceTab,

    // Secrets Manager
    pub secrets_filter: String,
    pub secrets_list: Vec<SecretEntry>,
    pub selected_secret: Option<usize>,
    pub secret_detail: Option<SecretValue>,
    pub secret_edit_buf: String,
    pub secret_dirty: bool,

    // Parameter Store
    pub params_filter: String,
    pub params_list: Vec<ParamEntry>,
    pub selected_param: Option<usize>,
    pub param_detail: Option<ParamValue>,
    pub param_edit_buf: String,
    pub param_dirty: bool,

    // Value caches (name → value)
    pub secret_cache: std::collections::HashMap<String, SecretValue>,
    pub param_cache: std::collections::HashMap<String, ParamValue>,

    // Create forms
    pub show_create_form: bool,
    pub new_secret_name: String,
    pub new_secret_value: String,
    pub new_param_name: String,
    pub new_param_value: String,
    pub new_param_type: String,

    // UI
    pub loading: bool,
    pub fetching_value: bool,
    pub status_message: Option<(String, StatusKind)>,
    pub error_dialog: Option<String>,
    pub delete_confirm: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            auth_mode: AuthMode::Profile,
            available_profiles: Vec::new(),
            selected_profile: 0,
            manual_creds: ManualCreds::default(),
            connected: false,
            active_tab: ServiceTab::ParameterStore,
            secrets_filter: String::new(),
            secrets_list: Vec::new(),
            selected_secret: None,
            secret_detail: None,
            secret_edit_buf: String::new(),
            secret_dirty: false,
            params_filter: String::new(),
            params_list: Vec::new(),
            selected_param: None,
            param_detail: None,
            param_edit_buf: String::new(),
            param_dirty: false,
            secret_cache: std::collections::HashMap::new(),
            param_cache: std::collections::HashMap::new(),
            show_create_form: false,
            new_secret_name: String::new(),
            new_secret_value: String::new(),
            new_param_name: String::new(),
            new_param_value: String::new(),
            new_param_type: "String".to_string(),
            loading: false,
            fetching_value: false,
            status_message: None,
            error_dialog: None,
            delete_confirm: None,
        }
    }
}
