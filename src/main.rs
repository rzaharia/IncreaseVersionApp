mod callback_validator;
use std::collections::HashMap;

use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use callback_validator::callback_validator;

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
    //TODO: replace log with trace
    init_logger().unwrap();
    let app = Router::new().route("/callback", post(callback_entrypoint));

    //add ip whitelisting https://api.github.com/meta
    //axum resource for whitelisting https://docs.rs/axum/latest/axum/struct.Router.html#method.into_make_service_with_connect_info
    let addr = "0.0.0.0:3000";
    info!("Started listening on addr: {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn callback_entrypoint(
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<User>) {
    info!("Got a callback!");
    let res = callback_validator(params, headers, payload).await;
    if let Err(err) = res {
        error!("Found error {err}");
        return (
            StatusCode::CREATED,
            Json(User {
                id: 2,
                username: "asd".to_string(),
            }),
        );
    }
    // let create_user: CreateUser = match serde_json::from_value(payload) {
    //     Ok(user) => user,
    //     Err(_) => {
    //         return (
    //             StatusCode::BAD_REQUEST,
    //             Json(User {
    //                 id: 0,
    //                 username: "".to_string(),
    //             }),
    //         )
    //     }
    // };
    info!("ALL GOOD");
    (
        StatusCode::CREATED,
        Json(User {
            id: 2,
            username: "asd".to_string(),
        }),
    )
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
