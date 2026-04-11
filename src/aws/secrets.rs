use aws_sdk_secretsmanager::Client;

use crate::state::{SecretEntry, SecretValue};

pub async fn list_secrets(client: &Client, prefix: &str) -> Result<Vec<SecretEntry>, String> {
    let mut entries = Vec::new();
    let mut next_token: Option<String> = None;

    loop {
        let mut req = client.list_secrets();
        if !prefix.is_empty() {
            req = req.filters(
                aws_sdk_secretsmanager::types::Filter::builder()
                    .key(aws_sdk_secretsmanager::types::FilterNameStringType::Name)
                    .values(prefix)
                    .build(),
            );
        }
        if let Some(token) = &next_token {
            req = req.next_token(token);
        }
        let resp = req.send().await.map_err(|e| e.to_string())?;
        if let Some(secrets) = resp.secret_list {
            for s in secrets {
                entries.push(SecretEntry {
                    name: s.name.unwrap_or_default(),
                    arn: s.arn.unwrap_or_default(),
                    description: s.description,
                });
            }
        }
        next_token = resp.next_token;
        if next_token.is_none() {
            break;
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub async fn get_secret_value(client: &Client, name: &str) -> Result<SecretValue, String> {
    let resp = client
        .get_secret_value()
        .secret_id(name)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(text) = resp.secret_string {
        Ok(SecretValue::Text(text))
    } else if let Some(blob) = resp.secret_binary {
        Ok(SecretValue::Binary(blob.into_inner()))
    } else {
        Err("Secret has no value".to_string())
    }
}

pub async fn create_secret(client: &Client, name: &str, value: &str) -> Result<String, String> {
    client
        .create_secret()
        .name(name)
        .secret_string(value)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Created secret '{}'", name))
}

pub async fn update_secret(client: &Client, name: &str, value: &str) -> Result<String, String> {
    client
        .update_secret()
        .secret_id(name)
        .secret_string(value)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Updated secret '{}'", name))
}

pub async fn delete_secret(client: &Client, name: &str) -> Result<String, String> {
    client
        .delete_secret()
        .secret_id(name)
        .force_delete_without_recovery(true)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Deleted secret '{}'", name))
}
