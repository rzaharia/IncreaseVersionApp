use std::fs;

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
        let Ok(given_time) =
            DateTime::parse_from_str(self.expires_at.as_str(), "%Y-%m-%dT%H:%M:%SZ")
        else {
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

pub fn read_installation_data(file_loc: &String) -> Option<InstallationTokenFileContent> {
    let Ok(data) = fs::read_to_string(file_loc) else {
        info!("Failed read installation file: `{file_loc}`");
        return None;
    };

    let Ok(file_content) = serde_json::from_slice::<InstallationTokenFileContent>(data.as_bytes())
    else {
        info!("Failed to parse installation file: `{file_loc}`");
        return None;
    };

    if !file_content.token_data.is_token_valid() {
        info!("Installation token has expired");
        return None;
    }

    Some(file_content)
}

pub fn write_file(file_loc: &String, token_content: InstallationTokenFileContent) -> Result<()> {
    let Ok(data) = serde_json::to_string(&token_content) else {
        bail!(AppErrors::InvalidDeserializationInstallationFile(
            file_loc.clone()
        ));
    };
    if let Err(err) = fs::write(file_loc, data) {
        bail!(AppErrors::FailedToSaveInstallationFile(err.to_string()));
    }
    Ok(())
}
