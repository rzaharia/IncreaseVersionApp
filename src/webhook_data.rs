#![allow(unused)]
use serde::Deserialize;

#[derive(Deserialize)]
pub struct WebHookRepositoryOwner {
    id: u128,
    name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct WebHookRepository {
    id: u128,
    name: String,
    full_name: String,
    owner: WebHookRepositoryOwner,
}

#[derive(Deserialize)]
pub struct WebHookPusher {
    name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct WebHookSender {
    login: String,
    id: u128,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Deserialize)]
pub struct WebHookInstallation {
    id: u128,
    node_id: String,
}

#[derive(Deserialize)]
pub struct WebHookCommitUser {
    name: String,
    email: String,
    username: String,
}

#[derive(Deserialize)]
pub struct WebHookCommit {
    id: String,
    tree_id: String,
    message: String,
    author: WebHookCommitUser,
    committer: WebHookCommitUser,
    added: Vec<String>,
    removed: Vec<String>,
    modified: Vec<String>,
}

#[derive(Deserialize)]
pub struct WebWebHook {
    #[serde(rename = "ref")]
    ref_: String,
    repository: WebHookRepository,
    pusher: WebHookPusher,
    sender: WebHookSender,
    installation: WebHookInstallation,
    commits: Vec<WebHookCommit>,
}
