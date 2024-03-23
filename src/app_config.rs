use std::env;

use anyhow::{bail, Result};

use crate::app_errors::AppErrors;
static WEBHOOK_OBSERVED_REF: &str = "refs/heads/main";
static WEBHOOK_COMMIT_TYPE_BOT: &str = "Bot";

static EXPECTED_ENV_VARS: [&str; 3] = [
    "CALLBACK_SECRET_TOKEN",
    "APP_NAME",
    "COMMIT_WHEN_SENDER_IS_BOT",
];

pub struct AppEnvVars {
    pub callback_token: String,
    pub app_name: String,
    pub commit_when_sender_is_bot: bool,
}

impl AppEnvVars {
    pub fn new() -> Result<AppEnvVars> {
        let mut result = AppEnvVars {
            app_name: String::new(),
            callback_token: String::new(),
            commit_when_sender_is_bot: false,
        };
        let mut missing_vars: Vec<&str> = Vec::with_capacity(EXPECTED_ENV_VARS.len());
        for var in EXPECTED_ENV_VARS {
            if let Ok(value) = env::var(var) {
                match var {
                    "CALLBACK_SECRET_TOKEN" => result.callback_token = value,
                    "APP_NAME" => result.app_name = value,
                    "CALLBACK_SECRET_TOKEN" => {
                        if let Ok(bool_value) = value.parse::<bool>(){
                            result.commit_when_sender_is_bot = bool_value;
                        }else{
                            missing_vars.push(var);
                        }
                    }
                    _ => {}
                }
                continue;
            }
            missing_vars.push(var);
        }
        if !missing_vars.is_empty(){
            let missing_vars_joined = missing_vars.join(", ");
            bail!(AppErrors::MissingEvironmentVariables(missing_vars_joined));
        }

        Ok(result)
    }
}
