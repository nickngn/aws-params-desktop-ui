use aws_config::SdkConfig;
use aws_credential_types::Credentials;
use aws_types::region::Region;

use crate::state::ManualCreds;

pub async fn from_profile(profile_name: &str) -> Result<SdkConfig, String> {
    let config = aws_config::from_env()
        .profile_name(profile_name)
        .load()
        .await;
    verify_connection(&config).await?;
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

    verify_connection(&config).await?;
    Ok(config)
}

/// Use STS GetCallerIdentity to verify credentials.
/// This API requires no special IAM permissions — any valid credential can call it.
async fn verify_connection(config: &SdkConfig) -> Result<(), String> {
    let sts_client = aws_sdk_sts::Client::new(config);
    sts_client
        .get_caller_identity()
        .send()
        .await
        .map_err(|e| format_sdk_error(&e))?;
    Ok(())
}

fn format_sdk_error<E: std::fmt::Debug, R: std::fmt::Debug>(
    err: &aws_sdk_sts::error::SdkError<E, R>,
) -> String {
    match err {
        aws_sdk_sts::error::SdkError::ServiceError(ctx) => {
            format!("{:?}", ctx.err())
        }
        aws_sdk_sts::error::SdkError::DispatchFailure(e) => {
            if e.is_io() {
                "Network error: could not reach AWS".to_string()
            } else if e.is_timeout() {
                "Connection timed out".to_string()
            } else {
                format!("Dispatch error: {e:?}")
            }
        }
        other => format!("{other}"),
    }
}
