use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use chrono::{DateTime, TimeZone, Utc};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::app_errors::AppErrors;

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

impl InstallationToken {
    pub fn is_token_valid(&self) -> bool {
        // Parse expiration time
        let Ok(given_time) = DateTime::parse_from_rfc3339(self.expires_at.as_str()) else {
            warn!("Failed to parse expires_at {}", self.expires_at);
            return false;
        };

        // Localize to UTC timezone
        let given_time_utc = Utc.from_utc_datetime(&given_time.naive_utc());
        return given_time_utc > Utc::now();
    }
}

#[derive(Serialize, Deserialize)]
pub struct InstallationTokenFileContent {
    pub token_data: InstallationToken,
}

static ISTALLATION_DATA_PATH: &str = "tokens";
pub fn create_token_folder() -> Result<()> {
    if !Path::new(ISTALLATION_DATA_PATH).exists() {
        fs::create_dir(ISTALLATION_DATA_PATH)?;
    }
    Ok(())
}

fn get_token_file_full_path(file_loc: &String) -> PathBuf {
    Path::new(ISTALLATION_DATA_PATH).join(file_loc)
}

pub fn read_installation_data(file_loc: &String) -> Option<InstallationTokenFileContent> {
    let full_location = get_token_file_full_path(file_loc);
    let Ok(data) = fs::read_to_string(full_location) else {
        info!("Failed read installation file: `{file_loc}`");
        return None;
    };

    let Ok(file_content) = serde_json::from_str::<InstallationTokenFileContent>(data.as_str())
    else {
        info!("Failed to parse installation file: `{file_loc}`");
        return None;
    };

    if !file_content.token_data.is_token_valid() {
        info!("Installation token has expired");
        return None;
    }

    info!("Installation token {file_loc} valid and loaded");
    Some(file_content)
}

pub fn save_installation_data(
    file_loc: &String,
    token_content: &InstallationTokenFileContent,
) -> Result<()> {
    let Ok(data) = serde_json::to_string(&token_content) else {
        bail!(AppErrors::InvalidDeserializationInstallationFile(
            file_loc.clone()
        ));
    };
    let full_location = get_token_file_full_path(file_loc);
    if let Err(err) = fs::write(full_location, data) {
        bail!(AppErrors::FailedToSaveInstallationFile(err.to_string()));
    }
    Ok(())
}
