use crate::{app_errors::AppErrors, webhook_data::WebWebHook};
use anyhow::{bail, ensure, Result};
use axum::{body::Bytes, http::HeaderMap};
use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::env;

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

async fn validate_headers_and_get_signature_256(headers: &HeaderMap) -> Result<String> {
    let mut signature_256: String = String::new();
    for header in EXPECTED_CALLBACK_HEADERS {
        let entry = headers.get(header);
        let Some(entry) = entry else {
            bail!(AppErrors::MissingHeader(header));
        };

        let Ok(val) = entry.to_str() else {
            bail!(AppErrors::HeaderInvalidFormatError(header));
        };

        match header {
            "X-Hub-Signature-256" => {
                ensure!(
                    val.starts_with("sha256="),
                    AppErrors::HeaderParsingError(header)
                );
                signature_256 = val.to_string();
            }
            "X-Hub-Signature" => ensure!(
                val.starts_with("sha1="),
                AppErrors::HeaderParsingError(header)
            ),
            "User-Agent" => ensure!(
                val.starts_with("GitHub-Hookshot/"),
                AppErrors::HeaderParsingError(header)
            ),
            "X-GitHub-Hook-ID" => ensure!(
                val.parse::<u32>().is_ok(),
                AppErrors::HeaderParsingError(header)
            ),
            "X-GitHub-Hook-Installation-Target-ID" => ensure!(
                val.parse::<u32>().is_ok(),
                AppErrors::HeaderParsingError(header)
            ),
            _ => {}
        }
    }
    Ok(signature_256)
}

async fn verify_signature(payload_body: &Bytes, signature: &str) -> Result<()> {
    ensure!(
        signature.len() >= 10,
        AppErrors::HeaderParsingError("X-Hub-Signature-256")
    );

    let signature_chracters = &signature[7..];
    let signature_size = signature_chracters.len();
    ensure!(
        signature_size % 2 == 0,
        AppErrors::SignatureError("Invalid header size")
    );

    let expected_signature = hex::decode(signature_chracters);
    let Ok(expected_signature) = expected_signature else {
        bail!(AppErrors::SignatureError("Invalid expected signature"));
    };

    let secret_token =
        env::var("CALLBACK_SECRET_TOKEN").expect("SECRET_TOKEN not found in environment variables");
    let hash_obj = HmacSha256::new_from_slice(secret_token.as_bytes());

    let Ok(mut hash_obj) = hash_obj else {
        bail!(AppErrors::SignatureError("Invalid hash obj"));
    };

    hash_obj.update(payload_body);

    let result = hash_obj.finalize().into_bytes();
    let result = result.as_slice();
    ensure!(
        result == expected_signature,
        AppErrors::SignatureError("Signatures do not match")
    );
    Ok(())
}

pub async fn callback_validator(
    query_params: HashMap<String, String>,
    headers: HeaderMap,
    payload: Bytes,
) -> Result<WebWebHook> {
    ensure!(
        query_params.is_empty(),
        AppErrors::TooManyQueryParams(query_params.len())
    );
    let signature_header = validate_headers_and_get_signature_256(&headers).await?;
    verify_signature(&payload, signature_header.as_str()).await?;

    let Ok(webhook) = serde_json::from_slice(&payload) else {
        bail!(AppErrors::InvalidPayload());
    };

    Ok(webhook)
}
