use crate::{installation_token_data::read_file, webhook_data::WebWebHook};

pub async fn increase_version(webhook: WebWebHook) -> Result<(), String> {
    let res = read_file().await?;

    Ok(())
}
