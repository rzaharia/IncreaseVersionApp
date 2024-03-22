use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use axum::{body::Bytes, http::HeaderMap};
use crate::{webhook_data::WebWebHook, worker::increase_version};

type HmacSha256 = Hmac<Sha256>;

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

static EXPECTED_HEADERS_INTEGER_VALUES: [&str; 2] =
    ["X-GitHub-Hook-ID", "X-GitHub-Hook-Installation-Target-ID"];

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
    for expected_value in EXPECTED_HEADERS_STARTING_VALUES {
        let value_data = headers.get(expected_value.0).unwrap().to_str().unwrap();
        if !value_data.starts_with(expected_value.1) {
            let header_name = expected_value.0;
            let expected_val = expected_value.1;
            return Err(format!(
                "Header{header_name}-{value_data} does not start with value {expected_val}!"
            ));
        }
    }

    for expected_integer in EXPECTED_HEADERS_INTEGER_VALUES {
        let value_data = headers.get(expected_integer).unwrap().to_str().unwrap();
        if value_data.parse::<u128>().is_err() {
            return Err(format!(
                "Header{expected_integer}-{value_data} does is a valid integer!"
            ));
        }
    }

    Ok(())
}

async fn verify_signature(payload_body: &Bytes, signature: &str) -> Result<(), String> {
    let signature_chracters = &signature[7..];
    let signature_size = signature_chracters.len();
    if signature_size % 2 != 0 {
        return Err("Invalid signature_chracters".to_string());
    }

    let expected_signatures = hex::decode(signature_chracters);
    if let Err(_) = expected_signatures {
        return Err("Failed to decode signature hash".to_string());
    }
    let expected_signature = expected_signatures.unwrap();

    let secret_token =
        env::var("CALLBACK_SECRET_TOKEN").expect("SECRET_TOKEN not found in environment variables");
    let hash_obj = HmacSha256::new_from_slice(secret_token.as_bytes());
    if let Err(err) = hash_obj {
        return Err(err.to_string());
    }
    let mut hash_obj = hash_obj.unwrap();
    hash_obj.update(payload_body);

    let result = hash_obj.finalize().into_bytes().to_vec();
    if result != expected_signature {
        return Err("Signature does not match!".to_string());
    }
    Ok(())
}

pub async fn callback_validator(
    query_params: HashMap<String, String>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<WebWebHook, String> {
    if !query_params.is_empty() {
        let params_count = query_params.len();
        return Err(format!("Found {params_count} instead of 0!"));
    }
    validate_headers(&headers).await?;
    let signature_header = headers
        .get("X-Hub-Signature-256")
        .unwrap()
        .to_str()
        .unwrap();

    verify_signature(&payload, signature_header).await?;

    let webhook: Result<WebWebHook, serde_json::Error> = serde_json::from_slice(&payload);
    if let Err(err) = webhook {
        return Err(format!("Found at parsing json: {err}!"));
    }

    Ok(webhook.unwrap())
}
