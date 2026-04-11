use configparser::ini::Ini;

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
