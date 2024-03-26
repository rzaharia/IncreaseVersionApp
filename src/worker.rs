use crate::{
    app_apis::{get_access_token, get_repo_file_content},
    app_config::AppEnvVars,
    app_errors::AppErrors,
    installation_token_data::{
        read_installation_data, save_installation_data, InstallationTokenFileContent,
    },
    webhook_data::WebWebHook,
};
use anyhow::{bail, Result};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{self, Algorithm, EncodingKey, Header};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Payload {
    iat: i64,
    exp: i64,
    iss: u128,
}

async fn create_jwt(env_vars: &AppEnvVars) -> Result<String> {
    let signing_key = &env_vars.private_signature;

    let Some(exp_minutes) = TimeDelta::try_minutes(10) else {
        bail!(AppErrors::FailedToProcessJWD(
            "Invalid jwt exp_minutes".to_string()
        ));
    };

    let iat = Utc::now().timestamp();
    let Some(exp) = Utc::now().checked_add_signed(exp_minutes) else {
        bail!(AppErrors::FailedToProcessJWD(
            "Failed jwt to add exp_minutes".to_string()
        ));
    };
    let exp = exp.timestamp();
    let payload = Payload {
        iss: env_vars.app_id,
        iat: iat,
        exp: exp,
    };

    // Encode JWT
    let header = Header::new(Algorithm::RS256);
    //let encoding_key = EncodingKey::from_secret(signing_key.as_ref());
    let Ok(encoding_key) = EncodingKey::from_rsa_pem(signing_key.as_ref()) else {
        bail!(AppErrors::FailedToProcessJWD(
            "Failed to process encoding_key".to_string()
        ));
    };
    let encoded_jwt = jsonwebtoken::encode(&header, &payload, &encoding_key);
    if let Err(err) = encoded_jwt {
        let err_text = format!("Failed jwt to encode {}:", err.to_string());
        bail!(AppErrors::FailedToProcessJWD(err_text));
    };

    Ok(encoded_jwt.unwrap())
}

pub async fn increase_version(env_vars: &AppEnvVars, webhook: WebWebHook) -> Result<()> {
    let file_name = format!("{}.json", webhook.installation.id);
    let mut current_installation: Option<InstallationTokenFileContent> =
        read_installation_data(&file_name);
    let mut token_needs_saving = false;
    if current_installation.is_none() {
        let jwt = create_jwt(&env_vars).await?;
        // let app_data = get_app_info(jwt.as_str()).await?;
        // info!(
        //     "Found app id:`{}`, slug:`{}`,name:{}",
        //     app_data.id, app_data.slug, app_data.name
        // );
        current_installation = Some(InstallationTokenFileContent {
            token_data: get_access_token(webhook.installation.id, jwt.as_str()).await?,
        });
        token_needs_saving = true;
    }

    let installation = current_installation.unwrap();
    if token_needs_saving {
        info!("Saved installation data {file_name}!");
        save_installation_data(&file_name, &installation)?;
    }

    let file = get_repo_file_content(
        &installation.token_data.token,
        &webhook.repository.owner.name,
        &webhook.repository.name,
        &env_vars.file_to_download,
        &env_vars.pattern_version_to_search
    )
    .await?;

    Ok(())
}
