use serde::Deserialize;

#[derive(Deserialize)]
pub struct WebHookRepositoryOwner {
    pub id: u128,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct WebHookRepository {
    pub id: u128,
    pub name: String,
    pub full_name: String,
    pub owner: WebHookRepositoryOwner,
}

#[derive(Deserialize)]
pub struct WebHookPusher {
    pub name: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct WebHookSender {
    pub login: String,
    pub id: u128,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Deserialize)]
pub struct WebHookInstallation {
    pub id: u128,
    pub node_id: String,
}

#[derive(Deserialize)]
pub struct WebHookCommitUser {
    pub name: String,
    pub email: String,
    pub username: String,
}

#[derive(Deserialize)]
pub struct WebHookCommit {
    pub id: String,
    pub tree_id: String,
    pub message: String,
    pub author: WebHookCommitUser,
    pub committer: WebHookCommitUser,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

#[derive(Deserialize)]
pub struct WebWebHook {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub repository: WebHookRepository,
    pub pusher: WebHookPusher,
    pub sender: WebHookSender,
    pub installation: WebHookInstallation,
    pub commits: Vec<WebHookCommit>,
}
