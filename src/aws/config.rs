use aws_config::SdkConfig;
use aws_credential_types::Credentials;
use aws_types::region::Region;

use crate::state::ManualCreds;

pub async fn from_profile(profile_name: &str) -> Result<SdkConfig, String> {
    let config = aws_config::from_env()
        .profile_name(profile_name)
        .load()
        .await;
    Ok(config)
}

pub async fn from_manual(creds: &ManualCreds) -> Result<SdkConfig, String> {
    if creds.access_key.is_empty() || creds.secret_key.is_empty() {
        return Err("Access Key and Secret Key are required".to_string());
    }
    if creds.region.is_empty() {
        return Err("Region is required for manual credentials".to_string());
    }

    let session = if creds.session_token.is_empty() {
        None
    } else {
        Some(creds.session_token.clone())
    };

    let credentials = Credentials::from_keys(&creds.access_key, &creds.secret_key, session);

    let config = aws_config::from_env()
        .credentials_provider(credentials)
        .region(Region::new(creds.region.clone()))
        .load()
        .await;

    Ok(config)
}
