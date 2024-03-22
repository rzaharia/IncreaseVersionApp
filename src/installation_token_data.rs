use std::{
    collections::HashMap,
    fmt::format,
    io::{Read, Write},
};

use log::error;
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::fs::OpenOptions;

#[derive(Serialize, Deserialize)]
pub struct InstallationTokenPermissions {
    pub contents: String,
    pub metadata: String,
}

#[derive(Serialize, Deserialize)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: String,
    pub permissions: InstallationTokenPermissions,
    pub repository_selection: String,
}
#[derive(Serialize, Deserialize)]
pub struct InstallationTokenFileContent {
    pub installations: HashMap<String, InstallationToken>,
}
static FILE_NAME: &str = "tokens_data.json";
pub async fn read_file() -> Result<InstallationTokenFileContent, String> {
    let file_res = OpenOptions::new().read(true).open(FILE_NAME).await;
    if let Err(_) = file_res {
        return Ok(InstallationTokenFileContent {
            installations: HashMap::new(),
        });
    }
    let mut file = file_res.unwrap().into_std().await;
    let data = tokio::task::spawn_blocking(move || {
        let mut s = String::new();
        if let Err(_) = file.read_to_string(&mut s) {
            error!("Failed to read file!");
            return Err("Failed to read file!");
        }
        return Ok(s);
    })
    .await;

    if let Err(err) = data {
        let err_data = format!("Failed to get data: {err}");
        return Err(err_data);
    }
    let data = data.unwrap();
    if let Err(err) = data {
        let err_data = format!("Failed to get data: {err}");
        return Err(err_data);
    }
    let data = data.unwrap();
    let file_content: Result<InstallationTokenFileContent, serde_json::Error> =
        serde_json::from_slice(data.as_bytes());
    if let Err(err) = file_content {
        let err_data = format!("Failed to get data: {err}");
        return Err(err_data);
    }

    Ok(file_content.unwrap())
}

pub async fn write_file(token_content: InstallationTokenFileContent) -> Result<(), String> {
    let file_res = OpenOptions::new().write(true).open(FILE_NAME).await;
    if let Err(err) = file_res {
        return Err(format!("{err}"));
    }
    let mut file = file_res.unwrap().into_std().await;
    let result = tokio::task::spawn_blocking(move || {
        let data = serde_json::to_string(&token_content);
        if let Err(err) = data {
            let err_string = format!("{err}");
            return Err(err_string);
        }
        let data = data.unwrap();
        if let Err(err) = file.write_all(data.as_bytes()) {
            let err_string = format!("{err}");
            return Err(err_string);
        }

        Ok(())
    })
    .await;

    if let Err(err) = result {
        let err_string = format!("{err}");
        return Err(err_string);
    }
    let result = result.unwrap();
    if let Err(err) = result {
        let err_string = format!("{err}");
        return Err(err_string);
    }
    Ok(())
}
