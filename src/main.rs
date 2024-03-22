mod app_errors;
mod callback_validator;
mod installation_token_data;
mod webhook_data;
mod worker;
extern crate dotenv;
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use callback_validator::callback_validator;
use core::panic;
use dotenv::dotenv;
use log::{error, info};
use std::{collections::HashMap, env};
use tokio::net::TcpListener;

use crate::worker::increase_version;

static WEBHOOK_OBSERVED_REF: &str = "refs/heads/main";
static WEBHOOK_COMMIT_TYPE_BOT: &str = "Bot";
static EXPECTED_ENV_VARS: [&str; 3] = [
    "CALLBACK_SECRET_TOKEN",
    "APP_NAME",
    "COMMIT_WHEN_SENDER_IS_BOT",
];

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
    //TODO: print a list of missing and neccesary vars and then panic!
    //TODO: pass env vars or add env vars to a global one
    for var in EXPECTED_ENV_VARS {
        env::var(var).expect(var);
    }
    //TODO: replace log with trace
    init_logger().unwrap();
    let app = Router::new().route("/callback", post(callback_entrypoint));

    //add ip whitelisting https://api.github.com/meta
    //axum resource for whitelisting https://docs.rs/axum/latest/axum/struct.Router.html#method.into_make_service_with_connect_info
    let addr = "0.0.0.0:3000";
    info!("Started listening on addr: {addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn callback_entrypoint_impl(
    params: HashMap<String, String>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<()> {
    //TODO: move all here
    //TODO: here just map err or OK value
    info!("Got a callback!");
    let webhook = callback_validator(params, headers, payload).await?;

    if webhook.ref_ != WEBHOOK_OBSERVED_REF {
        let found_ref = webhook.ref_;
        info!("Found other ref \"{found_ref}\" than observed one, will stop!");
        return Ok(());
    }

    if webhook.sender.type_ == WEBHOOK_COMMIT_TYPE_BOT {
        let commit_if_sender_bot = env::var("COMMIT_WHEN_SENDER_IS_BOT")
            .unwrap()
            .parse::<bool>();
        if let Err(err) = commit_if_sender_bot {
            panic!("Invalid var COMMIT_WHEN_SENDER_IS_BOT: {err}");
        }
        if !commit_if_sender_bot.unwrap() {
            info!("Found restriction onyl to commit when the sender is User, will stop here!");
            return Ok(());
        }

        let app_name = env::var("APP_NAME").expect("APP_NAME not found in environment variables");
        if webhook.sender.login == app_name {
            info!("The last commit was made by this bot, will ignore that one!");
            return Ok(());
        }
    }

    let result = increase_version(webhook).await?;

    info!("ALL GOOD");
    Ok(())
}

async fn callback_entrypoint(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    payload: Bytes,
) -> (StatusCode, String) {
    match callback_entrypoint_impl(params, headers, payload).await {
        Ok(()) => (StatusCode::OK, "OK".to_string()),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()),
    }
}
