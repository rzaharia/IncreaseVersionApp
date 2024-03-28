use anyhow::{bail, ensure, Result};
use axum::http::HeaderMap;
use base64::engine::{self};
use base64::{self, Engine as _};
use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, ClientBuilder, StatusCode};
use serde::Deserialize;
use serde_json::json;

use crate::app_errors::AppErrors;
use crate::installation_token_data::InstallationToken;

#[allow(dead_code)] //method used for testing if app is valid
#[derive(Deserialize)]
pub struct AuthenticatedAppData {
    pub id: u128,
    pub slug: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct FileConteAppDataApi {
    #[serde(rename = "type")]
    pub type_: String,
    pub encoding: String,
    pub size: u64,
    pub name: String,
    pub path: String,
    pub content: String,
}

pub struct FileConteAppDataDecoded {
    pub name: String,
    pub path: String,
    pub content: String,
    pub new_version: String,
}

#[derive(Deserialize)]
pub struct GithubTreeData {
    sha: String,
    //url:String,
}

#[derive(Deserialize)]
pub struct GithubCommitData {
    sha: String,
    //url:String,
}

impl FileConteAppDataApi {
    pub fn decode_file(&mut self) -> Result<()> {
        ensure!(
            self.encoding == "base64",
            AppErrors::FailedToDecodeFile("Not supported encoding format", self.encoding.clone())
        );
        let engine = engine::general_purpose::STANDARD;

        let mut buffer: Vec<u8> = Vec::with_capacity(self.size as usize);
        for line in self.content.split('\n') {
            engine.decode_vec(line.as_bytes(), &mut buffer)?;
        }
        self.content = String::from_utf8(buffer)?;
        Ok(())
    }

    pub fn increase_version(
        self,
        pattern_version_to_search: &String,
    ) -> Result<FileConteAppDataDecoded> {
        let Some(version_pos) = self.content.find(pattern_version_to_search) else {
            let err = format!("Could not find pattern: {}", pattern_version_to_search);
            bail!(AppErrors::FailedToIncreaseVersionInFile(err));
        };

        let mut endline_pos = self.content.as_str()[version_pos..].find('\n');
        if endline_pos.is_none() {
            endline_pos = Some(self.size as usize);
        }
        let mut endline_pos = endline_pos.unwrap();
        endline_pos += version_pos;

        let version_line = &self.content.as_str()[version_pos..endline_pos];
        let actual_version = &self.content.as_str()
            [version_pos + pattern_version_to_search.len()..endline_pos]
            .trim_matches(|c| c == ' ' || c == '"');

        let mut version_split: Vec<&str> = actual_version.split('.').collect();
        ensure!(
            version_split.len() == 3,
            AppErrors::FailedToIncreaseVersionInFile(
                "Failed to obtain version in format: MAJOR.MINOR.PATCH".to_string()
            )
        );
        let temp_mid_string = (version_split[1].parse::<u32>()? + 1).to_string();
        version_split[1] = temp_mid_string.as_str();

        let version_split = version_split.join(".");
        let final_version = format!("{} \"{}\"", pattern_version_to_search, version_split);
        let new_content = self.content.replace(version_line, &final_version);

        let result = FileConteAppDataDecoded {
            name: self.name,
            path: self.path,
            content: new_content,
            new_version: version_split,
        };

        Ok(result)
    }
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

pub async fn get_repo_file_content_impl(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    file_path: &String,
) -> Result<FileConteAppDataApi, reqwest::Error> {
    let client = get_client_with_default_headers(token)?;
    let link =
        format!("https://api.github.com/repos/{repo_owner}/{repo_name}/contents/{file_path}");
    let response = client.get(link).send().await?;

    let data = response.json::<FileConteAppDataApi>().await?;
    Ok(data)
}

//https://docs.github.com/en/rest/repos/contents?apiVersion=2022-11-28
pub async fn get_repo_file_content(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    file_path: &String,
    pattern_version_to_search: &String,
) -> Result<FileConteAppDataDecoded> {
    match get_repo_file_content_impl(token, repo_owner, repo_name, file_path).await {
        Ok(mut result) => {
            result.decode_file()?;
            let decoded_data = result.increase_version(&pattern_version_to_search)?;
            return Ok(decoded_data);
        }
        Err(err) => bail!(AppErrors::ApiFailure(
            "get_access_token",
            err.without_url().to_string()
        )),
    }
}

async fn create_tree_impl(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    base_tree: &String,
    file_content: &FileConteAppDataDecoded,
) -> Result<(GithubTreeData, StatusCode), reqwest::Error> {
    let body_data = json!({
        "base_tree": base_tree,
        "tree": [
            {
                "path": file_content.path,
                "mode": "100644",
                "type": "blob",
                "content": file_content.content
            }
        ]
    });

    let client = get_client_with_default_headers(token)?;
    let link = format!("https://api.github.com/repos/{repo_owner}/{repo_name}/git/trees");
    let response = client.post(link).json(&body_data).send().await?;
    let status_code = response.status();

    let data = response.json::<GithubTreeData>().await?;
    Ok((data, status_code))
}

// https://docs.github.com/en/rest/git/trees?apiVersion=2022-11-28
pub async fn create_tree(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    base_tree: &String,
    file_content: &FileConteAppDataDecoded,
) -> Result<GithubTreeData> {
    match create_tree_impl(token, repo_owner, repo_name, base_tree, file_content).await {
        Ok((result, status_code)) => {
            if status_code != StatusCode::CREATED {
                let err_msg =
                    format!("Failed to create tree, expectected status 201 and got {status_code}");
                bail!(AppErrors::ApiFailure("create_tree", err_msg));
            }
            return Ok(result);
        }
        Err(err) => bail!(AppErrors::ApiFailure(
            "create_tree",
            err.without_url().to_string()
        )),
    }
}

async fn create_commit_impl(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    commit: &String,
    file_content: &FileConteAppDataDecoded,
    tree_data: &GithubTreeData,
) -> Result<(GithubCommitData, StatusCode), reqwest::Error> {
    let client = get_client_with_default_headers(token)?;

    let body_data = json!({
        "message": format!("Increase version to {}",file_content.new_version),
        "parents": [commit],
        "tree": tree_data.sha,
    });

    let link = format!("https://api.github.com/repos/{repo_owner}/{repo_name}/git/commits");
    let response = client.post(link).json(&body_data).send().await?;
    let status_code = response.status();

    let data = response.json::<GithubCommitData>().await?;
    Ok((data, status_code))
}

//https://docs.github.com/en/rest/git/commits
pub async fn create_commit(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    commit: &String,
    file_content: &FileConteAppDataDecoded,
    tree_data: &GithubTreeData,
) -> Result<GithubCommitData> {
    match create_commit_impl(
        token,
        repo_owner,
        repo_name,
        commit,
        file_content,
        tree_data,
    )
    .await
    {
        Ok((result, status_code)) => {
            if status_code != StatusCode::CREATED {
                let err_msg = format!(
                    "Failed to create commit, expectected status 201 and got {status_code}"
                );
                bail!(AppErrors::ApiFailure("create_commit", err_msg));
            }
            return Ok(result);
        }
        Err(err) => bail!(AppErrors::ApiFailure(
            "create_commit",
            err.without_url().to_string()
        )),
    }
}

async fn update_a_refence_impl(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    commit_data: &GithubCommitData,
    ref_to_use: &String,
) -> Result<(), reqwest::Error> {
    let client = get_client_with_default_headers(token)?;

    let body_data = json!({
        "sha": commit_data.sha,
        "force": true
    });

    let link = format!("https://api.github.com/repos/{repo_owner}/{repo_name}/git/{ref_to_use}");
    let _ = client.post(link).json(&body_data).send().await?;
    Ok(())
}

//https://docs.github.com/en/rest/git/refs?apiVersion=2022-11-28
pub async fn update_a_refence(
    token: &str,
    repo_owner: &String,
    repo_name: &String,
    commit_data: &GithubCommitData,
    ref_to_use: &String,
) -> Result<()> {
    match update_a_refence_impl(token, repo_owner, repo_name, commit_data, ref_to_use).await {
        Ok(()) => {
            return Ok(());
        }
        Err(err) => bail!(AppErrors::ApiFailure(
            "update_a_refence",
            err.without_url().to_string()
        )),
    }
}
