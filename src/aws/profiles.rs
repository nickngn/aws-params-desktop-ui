use configparser::ini::Ini;

use crate::state::ManualCreds;

pub fn list_profiles() -> Vec<String> {
    let mut profiles = Vec::new();

    if let Some(home) = dirs::home_dir() {
        // Parse ~/.aws/credentials
        let creds_path = home.join(".aws").join("credentials");
        if creds_path.exists() {
            let mut ini = Ini::new();
            if ini.load(creds_path.to_string_lossy().as_ref()).is_ok() {
                for section in ini.sections() {
                    if section != "DEFAULT" && !profiles.contains(&section) {
                        profiles.push(section);
                    }
                }
            }
        }

        // Parse ~/.aws/config
        let config_path = home.join(".aws").join("config");
        if config_path.exists() {
            let mut ini = Ini::new();
            if ini.load(config_path.to_string_lossy().as_ref()).is_ok() {
                for section in ini.sections() {
                    let name = section
                        .strip_prefix("profile ")
                        .unwrap_or(&section)
                        .to_string();
                    if name != "DEFAULT" && !profiles.contains(&name) {
                        profiles.push(name);
                    }
                }
            }
        }
    }

    // Ensure "default" is first
    if let Some(pos) = profiles.iter().position(|p| p == "default") {
        profiles.remove(pos);
        profiles.insert(0, "default".to_string());
    }

    if profiles.is_empty() {
        profiles.push("default".to_string());
    }

    profiles
}

/// Read access_key, secret_key, session_token from ~/.aws/credentials
/// and region from ~/.aws/config for the given profile name.
pub fn read_profile_creds(profile_name: &str) -> ManualCreds {
    let mut creds = ManualCreds::default();

    let Some(home) = dirs::home_dir() else {
        return creds;
    };

    // Read keys from ~/.aws/credentials [profile_name]
    let creds_path = home.join(".aws").join("credentials");
    if creds_path.exists() {
        let mut ini = Ini::new();
        if ini.load(creds_path.to_string_lossy().as_ref()).is_ok() {
            if let Some(val) = ini.get(profile_name, "aws_access_key_id") {
                creds.access_key = val;
            }
            if let Some(val) = ini.get(profile_name, "aws_secret_access_key") {
                creds.secret_key = val;
            }
            if let Some(val) = ini.get(profile_name, "aws_session_token") {
                creds.session_token = val;
            }
            // Some credentials files also contain region
            if let Some(val) = ini.get(profile_name, "region") {
                creds.region = val;
            }
        }
    }

    // Read region from ~/.aws/config [profile <name>] or [default]
    let config_path = home.join(".aws").join("config");
    if config_path.exists() {
        let mut ini = Ini::new();
        if ini.load(config_path.to_string_lossy().as_ref()).is_ok() {
            // In config file, default profile is [default], others are [profile <name>]
            let section = if profile_name == "default" {
                "default".to_string()
            } else {
                format!("profile {}", profile_name)
            };
            if let Some(val) = ini.get(&section, "region") {
                creds.region = val;
            }
        }
    }

    creds
}
