use crate::{
    installation_token_data::{read_installation_data, write_file, InstallationTokenFileContent},
    webhook_data::WebWebHook,
};
use anyhow::Result;
use log::{error, info};

pub async fn increase_version(webhook: WebWebHook) -> Result<()> {
    let file_name = format!("{}.json", webhook.installation.id);
    let mut current_installation: Option<InstallationTokenFileContent> = read_installation_data(&file_name);

    Ok(())
}
