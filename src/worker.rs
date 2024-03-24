use crate::{
    app_config::AppEnvVars,
    app_errors::AppErrors,
    installation_token_data::{read_installation_data, write_file, InstallationTokenFileContent},
    webhook_data::WebWebHook,
};
use anyhow::{bail, Result};
use chrono::{Duration, TimeDelta, Utc};
use jsonwebtoken::{self, Algorithm, EncodingKey, Header};
use log::{error, info};
use serde::{Deserialize, Serialize};

/// Our claims struct, it needs to derive `Serialize` and/or `Deserialize`
#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    iat: i64,
    exp: i64,
    iss: u128,
}

async fn create_jwt(env_vars: &AppEnvVars) -> Result<String> {
    let signing_key = &env_vars.private_signature;

    let Some(exp_minutes) = TimeDelta::try_minutes(10) else {
        bail!(AppErrors::FailedToProcessJWD("Invalid exp_minutes"));
    };

    let iat = Utc::now().timestamp();
    let Some(exp) = Utc::now().checked_add_signed(exp_minutes) else {
        bail!(AppErrors::FailedToProcessJWD("Failed to add exp_minutes"));
    };
    let exp = exp.timestamp();
    let payload = Payload {
        iss: env_vars.app_id,
        iat: iat,
        exp: exp,
    };

    // Encode JWT
    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_secret(signing_key.as_ref());
    let encoded_jwt = jsonwebtoken::encode(&header, &payload, &encoding_key).unwrap();

    Ok(encoded_jwt)
}

pub async fn increase_version(env_vars: &AppEnvVars, webhook: WebWebHook) -> Result<()> {
    let file_name = format!("{}.json", webhook.installation.id);
    let mut current_installation: Option<InstallationTokenFileContent> =
        read_installation_data(&file_name);
    if current_installation.is_none() {
        let jwt = create_jwt(&env_vars).await?;
    }

    Ok(())
}
