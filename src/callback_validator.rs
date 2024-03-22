use std::{collections::HashMap, fmt::format};

use axum::{
    http::{HeaderMap, StatusCode},
    Json,
};
use serde_json::Value;

use crate::{CreateUser, User};

static EXPECTED_CALLBACK_HEADERS: [&str; 8] = [
    "X-GitHub-Hook-ID",
    "X-GitHub-Event",
    "X-GitHub-Delivery",
    "X-Hub-Signature",
    "X-Hub-Signature-256",
    "User-Agent",
    "X-GitHub-Hook-Installation-Target-Type",
    "X-GitHub-Hook-Installation-Target-ID",
];

static EXPECTED_HEADERS_STARTING_VALUES: [(&'static str, &'static str); 3] = [
    ("X-Hub-Signature-256", "sha256="),
    ("X-Hub-Signature", "sha1="),
    ("User-Agent", "GitHub-Hookshot/"),
];

static EXPECTED_HEADERS_INTEGER_VALUES: [&str;2] = [
    "X-GitHub-Hook-ID",
    "X-GitHub-Hook-Installation-Target-ID"
];

pub struct WebHookOK {}

async fn validate_headers(headers: &HeaderMap) -> Result<(), String> {
    for header in EXPECTED_CALLBACK_HEADERS {
        let entry = headers.get(header);
        if entry.is_none() {
            return Err(format!("Missing header {}!", header));
        }
        if entry.unwrap().to_str().is_err() {
            return Err(format!("Header {} is not ASCII!", header));
        }
    }
    for expected_value in EXPECTED_HEADERS_STARTING_VALUES{
        let value_data = headers.get(expected_value.0).unwrap().to_str().unwrap();
        if !value_data.starts_with(expected_value.1){
            let header_name = expected_value.0;
            let expected_val = expected_value.1;
            return Err(format!("Header{header_name}-{value_data} does not start with value {expected_val}!"));
        }
    }

    for expected_integer in EXPECTED_HEADERS_INTEGER_VALUES {
        let value_data = headers.get(expected_integer).unwrap().to_str().unwrap();
        if value_data.parse::<u128>().is_err(){
            return Err(format!("Header{expected_integer}-{value_data} does is a valid integer!"));
        }
    }
    
    Ok(())
}

pub async fn callback_validator(
    query_params: HashMap<String, String>,
    headers: HeaderMap,
    payload: Value,
) -> Result<WebHookOK, String> {
    if !query_params.is_empty() {
        let params_count = query_params.len();
        return Err(format!("Found {params_count} instead of 0!"));
    }
    validate_headers(&headers).await?;
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
    Ok(WebHookOK {})
}
