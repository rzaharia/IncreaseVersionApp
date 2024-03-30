use std::{env, fs, path::Path};

use anyhow::{bail, ensure, Result};
use log::info;
use serde::{Deserialize, Serialize};

use crate::{app_errors::AppErrors, installation_token_data::create_token_folder};
pub static WEBHOOK_OBSERVED_REF: &str = "refs/heads/main";
pub static WEBHOOK_COMMIT_TYPE_BOT: &str = "Bot";
pub static CONFIG_FILE_APP: &str = "IncreaseAppVersion.json";

static EXPECTED_ENV_VARS: [&str; 7] = [
    "CALLBACK_SECRET_TOKEN",
    "APP_NAME",
    "PRIVATE_KEY_FILE_LOC",
    "APP_ID",
    "COMMIT_WHEN_SENDER_IS_BOT",
    "FILE_TO_DOWNLOAD",
    "PATTERN_VERSION_TO_SEARCH",
];

static CONFIG_DATA_PATH: &str = "config";
pub fn create_app_folder() -> Result<()> {
    if !Path::new(CONFIG_DATA_PATH).exists() {
        fs::create_dir(CONFIG_DATA_PATH)?;
    }
    create_token_folder()?;
    Ok(())
}

#[derive(Clone, Serialize, Deserialize)] //Clone needed by axum state
pub struct AppConfig {
    pub callback_token: String,
    pub app_name: String,
    pub private_signature: String,
    pub app_id: u128,

    pub commit_when_sender_is_bot: bool,
    pub file_to_download: String,
    pub pattern_version_to_search: String,
    pub branch_refs_to_observe: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub commit_when_sender_is_bot: bool,
    pub file_to_donwload: String,
    pub pattern_version_to_search: String,
    pub branch_refs_to_observe: Vec<String>,
}

fn try_read_file(file_loc: &String) -> Result<String> {
    let data = fs::read_to_string(file_loc)?;
    ensure!(
        !data.is_empty(),
        AppErrors::InvalidEvironmentVariable(file_loc.to_string(), "file empty")
    );
    Ok(data)
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            callback_token: Default::default(),
            app_name: "IncreaseAppVersion".to_string(),
            private_signature: "sig.pem".to_string(),
            app_id: Default::default(),
            commit_when_sender_is_bot: false,
            file_to_download: "version.hpp".to_string(),
            pattern_version_to_search: "#define VERSION".to_string(),
            branch_refs_to_observe: ["refs/heads/main".to_string()].to_vec(),
        }
    }
}

impl AppConfig {
    fn generate_default_config() -> AppConfig {
        let config = AppConfig::default();
        let data = serde_json::to_string(&config).expect("failed to convert AppConfig");

        let file_path = Path::new(CONFIG_DATA_PATH).join(CONFIG_FILE_APP);
        let file_path = file_path.to_str().expect("invalid location");
        fs::write(file_path, data).expect("coudl not write CONFIG_DATA_PATH");

        config
    }

    fn read_config_file() -> Result<AppConfig> {
        let file_path = Path::new(CONFIG_DATA_PATH).join(CONFIG_FILE_APP);
        let file_path = file_path.to_str().expect("invalid location");
        let file_data = try_read_file(&file_path.to_string())?;
        let app_config = serde_json::from_str::<AppConfig>(&file_data.as_str())?;

        Ok(app_config)
    }

    pub fn new() -> Result<AppConfig> {
        let Ok(mut result) = Self::read_config_file() else {
            return Ok(Self::generate_default_config());
        };
        if let Err(err) = Self::read_vars_from_env(&mut result) {
            info!("Failed to read env vars: {err}");
        }
        Ok(result)
    }

    fn read_vars_from_env(result: &mut AppConfig) -> Result<()> {
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
                    "PATTERN_VERSION_TO_SEARCH" => result.pattern_version_to_search = value,
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

        Ok(())
    }
}
