use std::{env, fs};

use anyhow::{bail, ensure, Result};

use crate::app_errors::AppErrors;
pub static WEBHOOK_OBSERVED_REF: &str = "refs/heads/main";
pub static WEBHOOK_COMMIT_TYPE_BOT: &str = "Bot";

static EXPECTED_ENV_VARS: [&str; 6] = [
    "CALLBACK_SECRET_TOKEN",
    "APP_NAME",
    "COMMIT_WHEN_SENDER_IS_BOT",
    "PRIVATE_KEY_FILE_LOC",
    "APP_ID",
    "FILE_TO_DOWNLOAD",
];

#[derive(Clone, Default)] //Clone needed by axum state
pub struct AppEnvVars {
    pub callback_token: String,
    pub app_name: String,
    pub commit_when_sender_is_bot: bool,
    pub private_signature: String,
    pub app_id: u128,
    pub file_to_download: String,
}

fn try_read_file(file_loc: &String) -> Result<String> {
    let data = fs::read_to_string(file_loc)?;
    ensure!(
        !data.is_empty(),
        AppErrors::InvalidEvironmentVariable(file_loc.to_string(), "file empty")
    );
    Ok(data)
}

impl AppEnvVars {
    pub fn new() -> Result<AppEnvVars> {
        let mut result = AppEnvVars::default();
        let mut missing_vars: Vec<&str> = Vec::with_capacity(EXPECTED_ENV_VARS.len());
        for var in EXPECTED_ENV_VARS {
            if let Ok(value) = env::var(var) {
                match var {
                    "CALLBACK_SECRET_TOKEN" => result.callback_token = value,
                    "APP_NAME" => result.app_name = value,
                    "COMMIT_WHEN_SENDER_IS_BOT" => {
                        if let Ok(bool_value) = value.parse::<bool>() {
                            result.commit_when_sender_is_bot = bool_value;
                        } else {
                            bail!(AppErrors::InvalidEvironmentVariable(
                                var.to_string(),
                                "invalid value"
                            ));
                        }
                    }
                    "PRIVATE_KEY_FILE_LOC" => {
                        let Ok(signature) = try_read_file(&value) else {
                            bail!(AppErrors::InvalidEvironmentVariable(
                                var.to_string(),
                                "could not read file"
                            ));
                        };
                        result.private_signature = signature;
                    }
                    "APP_ID" => {
                        if let Ok(value) = value.parse::<u128>() {
                            result.app_id = value;
                        } else {
                            bail!(AppErrors::InvalidEvironmentVariable(
                                var.to_string(),
                                "invalid value, not unsinged number"
                            ));
                        }
                    }
                    "FILE_TO_DOWNLOAD" => result.file_to_download = value,
                    _ => {}
                }
                continue;
            }
            missing_vars.push(var);
        }
        if !missing_vars.is_empty() {
            let missing_vars_joined = missing_vars.join(", ");
            bail!(AppErrors::MissingEvironmentVariables(missing_vars_joined));
        }

        Ok(result)
    }
}
