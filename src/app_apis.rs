use anyhow::{bail, Result};
use axum::http::HeaderMap;
use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, ClientBuilder};
use serde::Deserialize;

use crate::app_errors::AppErrors;
use crate::installation_token_data::InstallationToken;

#[allow(dead_code)] //method used for testing if app is valid
#[derive(Deserialize)]
pub struct AuthenticatedAppData {
    pub id: u128,
    pub slug: String,
    pub name: String,
}

fn get_client_with_default_headers(jwt_token: &str) -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("IncreaseVersionAPP/0.1.0"),
    );
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", jwt_token)).unwrap(),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );

    let client = ClientBuilder::new().default_headers(headers).build()?;
    Ok(client)
}

//Get the authenticated app from https://docs.github.com/en/rest/apps/apps?apiVersion=2022-11-28#get-the-authenticated-app
async fn get_app_info_impl(jwt_token: &str) -> Result<AuthenticatedAppData, reqwest::Error> {
    let client = get_client_with_default_headers(jwt_token)?;
    let response = client.get("https://api.github.com/app").send().await?;

    let data = response.json::<AuthenticatedAppData>().await?;

    Ok(data)
}

#[allow(dead_code)] //method used for testing if app is valid
pub async fn get_app_info(jwt_token: &str) -> Result<AuthenticatedAppData> {
    match get_app_info_impl(jwt_token).await {
        Ok(result) => return Ok(result),
        Err(err) => bail!(AppErrors::ApiFailure(
            "get_app_info_impl",
            err.without_url().to_string()
        )),
    }
}

pub async fn get_access_token_impl(
    installation_id: u128,
    jwt_token: &str,
) -> Result<InstallationToken, reqwest::Error> {
    let client = get_client_with_default_headers(jwt_token)?;
    let link = format!("https://api.github.com/app/installations/{installation_id}/access_tokens");
    let response = client.post(link).send().await?;

    let data = response.json::<InstallationToken>().await?;

    Ok(data)
}

//https://docs.github.com/en/rest/apps/apps?apiVersion=2022-11-28#get-an-installation-for-the-authenticated-app
pub async fn get_access_token(installation_id: u128, jwt_token: &str) -> Result<InstallationToken> {
    match get_access_token_impl(installation_id, jwt_token).await {
        Ok(result) => return Ok(result),
        Err(err) => bail!(AppErrors::ApiFailure(
            "get_access_token",
            err.without_url().to_string()
        )),
    }
}
