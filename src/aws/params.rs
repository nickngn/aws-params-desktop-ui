use aws_sdk_ssm::Client;
use aws_sdk_ssm::types::ParameterType;

use crate::state::{ParamEntry, ParamValue};

pub async fn list_parameters(client: &Client, prefix: &str) -> Result<Vec<ParamEntry>, String> {
    let mut entries = Vec::new();

    if prefix.is_empty() {
        let mut next_token: Option<String> = None;
        loop {
            let mut req = client.describe_parameters();
            if let Some(token) = &next_token {
                req = req.next_token(token);
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            if let Some(params) = resp.parameters {
                for p in params {
                    entries.push(ParamEntry {
                        name: p.name.unwrap_or_default(),
                        param_type: p.r#type
                            .map(|t| format!("{}", t.as_str()))
                            .unwrap_or_default(),
                        version: p.version,
                    });
                }
            }
            next_token = resp.next_token;
            if next_token.is_none() {
                break;
            }
        }
    } else {
        let mut next_token: Option<String> = None;
        loop {
            let mut req = client
                .get_parameters_by_path()
                .path(prefix)
                .recursive(true)
                .with_decryption(true);
            if let Some(token) = &next_token {
                req = req.next_token(token);
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            if let Some(params) = resp.parameters {
                for p in params {
                    entries.push(ParamEntry {
                        name: p.name.unwrap_or_default(),
                        param_type: p.r#type
                            .map(|t| format!("{}", t.as_str()))
                            .unwrap_or_default(),
                        version: p.version,
                    });
                }
            }
            next_token = resp.next_token;
            if next_token.is_none() {
                break;
            }
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

pub async fn get_parameter(client: &Client, name: &str) -> Result<ParamValue, String> {
    let resp = client
        .get_parameter()
        .name(name)
        .with_decryption(true)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let param = resp.parameter.ok_or("Parameter not found")?;
    Ok(ParamValue {
        text: param.value.unwrap_or_default(),
        param_type: param.r#type
            .map(|t| format!("{}", t.as_str()))
            .unwrap_or_default(),
    })
}

pub async fn create_parameter(
    client: &Client,
    name: &str,
    value: &str,
    param_type: &str,
) -> Result<String, String> {
    let pt = match param_type {
        "SecureString" => ParameterType::SecureString,
        _ => ParameterType::String,
    };

    client
        .put_parameter()
        .name(name)
        .value(value)
        .r#type(pt)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Created parameter '{}'", name))
}

pub async fn update_parameter(
    client: &Client,
    name: &str,
    value: &str,
) -> Result<String, String> {
    client
        .put_parameter()
        .name(name)
        .value(value)
        .overwrite(true)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Updated parameter '{}'", name))
}

pub async fn delete_parameter(client: &Client, name: &str) -> Result<String, String> {
    client
        .delete_parameter()
        .name(name)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Deleted parameter '{}'", name))
}
