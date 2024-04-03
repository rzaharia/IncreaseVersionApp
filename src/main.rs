mod app_apis;
mod app_config;
mod app_errors;
mod callback_validator;
mod installation_token_data;
mod webhook_data;
mod worker;
extern crate dotenv;
use crate::{
    app_apis::get_github_environment_details,
    app_config::{create_app_folder, AppConfig, RepositoryConfig, WEBHOOK_COMMIT_TYPE_BOT},
    worker::increase_version,
};
use anyhow::Result;
use app_config::SecurityConfig;
use axum::{
    body::Bytes,
    extract::{ConnectInfo, Extension, Query, State},
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use callback_validator::callback_validator;
use core::panic;
use dotenv::dotenv;
use log::{error, info};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init_logger() -> Result<(), log::SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info))
}

#[tokio::main]
async fn main() {
    //TODO: follow best practices https://docs.github.com/en/webhooks/using-webhooks/best-practices-for-using-webhooks
    dotenv().ok();
    let app_config_res = AppConfig::new();
    if let Err(err) = app_config_res {
        let err_string = err.to_string();
        panic!("Invalid environment variables: {err_string}");
    }
    let app_config = app_config_res.unwrap();

    if let Err(err) = create_app_folder() {
        panic!("Failed to create app folders: {err}");
    }

    //TODO: replace log with trace
    init_logger().unwrap();

    let security_details = get_github_environment_details().await;
    if let Err(err) = security_details {
        error!("failed to obtain security settings: {err}");
        return;
    }
    let security_details = Arc::new(security_details.unwrap());

    let app = Router::new()
        .route("/callback", post(callback_entrypoint))
        .with_state(app_config)
        .layer(Extension(security_details))
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = "0.0.0.0:3000";
    info!("Started listening on addr: {addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn callback_entrypoint_impl(
    app_config: AppConfig,
    params: HashMap<String, String>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<()> {
    info!("Got a callback!");
    let webhook = callback_validator(&app_config, params, headers, payload).await?;

    let file_name = format!("{}.json", webhook.installation.id);
    let repo_config = RepositoryConfig::new(file_name.as_str(), &app_config)?;

    if !repo_config.branch_refs_to_observe.contains(&webhook.ref_) {
        let found_ref = webhook.ref_;
        info!("Found other ref \"{found_ref}\" than observed one, will stop!");
        return Ok(());
    }

    if webhook.sender.type_ == WEBHOOK_COMMIT_TYPE_BOT {
        if !repo_config.commit_when_sender_is_bot {
            info!("Found restriction onyl to commit when the sender is User, will stop here!");
            return Ok(());
        }

        if webhook.sender.login == app_config.app_name {
            info!("The last commit was made by this bot, will ignore that one!");
            return Ok(());
        }
    }

    increase_version(&app_config, &repo_config, webhook).await?;

    info!("ALL GOOD");
    Ok(())
}

async fn callback_entrypoint(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(env_vars): State<AppConfig>,
    Extension(security): Extension<Arc<SecurityConfig>>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    payload: Bytes,
) -> (StatusCode, String) {
    if !env_vars.whitelist_ips.contains(&addr.ip().to_string()) && !security.contains(addr.ip()) {
        error!(
            "Invalid ip {} conenected, will be blocked!",
            addr.ip().to_string()
        );
        return (StatusCode::FORBIDDEN, "Invalid".to_string());
    }
    match callback_entrypoint_impl(env_vars, params, headers, payload).await {
        Ok(()) => (StatusCode::OK, "OK".to_string()),
        Err(err) => {
            info!("Failed: {}", err.to_string());
            (StatusCode::BAD_REQUEST, err.to_string())
        }
    }
}
