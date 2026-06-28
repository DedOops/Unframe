use bytes::Bytes;
use chrono::Utc;
use reqwest::{Client, ClientBuilder, StatusCode};
use std::time::Duration;
use thiserror::Error;
use tracing::{info, warn};
use url::Url;

use unframe_model::NetworkAudit;

const MAX_RESPONSE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
const MAX_REDIRECTS: usize = 5;
const TIMEOUT_SECS: u64 = 30;
const USER_AGENT: &str = "Unframe/0.1 (document-client)";

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("response too large (exceeded {MAX_RESPONSE_BYTES} bytes)")]
    ResponseTooLarge,
    #[error("too many redirects")]
    TooManyRedirects,
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("HTTP error {0}")]
    HttpStatus(StatusCode),
    #[error("timeout")]
    Timeout,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub requested_url: Url,
    pub final_url: Url,
    pub content_type: String,
    pub content_type_raw: String,
    pub body: Bytes,
    pub audit: NetworkAudit,
    pub retrieved_at: chrono::DateTime<Utc>,
}

pub struct NetworkGateway {
    client: Client,
}

impl NetworkGateway {
    pub fn new() -> Result<Self, NetworkError> {
        let client = ClientBuilder::new()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .redirect(reqwest::redirect::Policy::limited(MAX_REDIRECTS))
            .https_only(false)
            .no_proxy()
            .build()?;
        Ok(Self { client })
    }

    pub async fn fetch(&self, url: Url) -> Result<FetchResult, NetworkError> {
        info!("fetching {}", url);
        let retrieved_at = Utc::now();

        let response = self.client.get(url.clone()).send().await.map_err(|e| {
            if e.is_timeout() {
                NetworkError::Timeout
            } else if e.is_redirect() {
                NetworkError::TooManyRedirects
            } else {
                NetworkError::Request(e)
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            warn!("HTTP {} for {}", status, url);
            return Err(NetworkError::HttpStatus(status));
        }

        let final_url = response.url().clone();
        let redirect_count = if final_url != url { 1 } else { 0 }; // simplified

        let content_type_raw = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let content_type = content_type_raw
            .split(';')
            .next()
            .unwrap_or("application/octet-stream")
            .trim()
            .to_string();

        // Stream body with size limit
        let mut body = Vec::new();
        let raw_bytes = response.bytes().await?;

        if raw_bytes.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(NetworkError::ResponseTooLarge);
        }
        body.extend_from_slice(&raw_bytes);

        let total_bytes = body.len() as u64;
        let audit = NetworkAudit {
            total_requests: 1,
            total_bytes,
            redirect_count,
            blocked_count: 0,
            scripts_discovered: 0,
            scripts_executed: 0,
            third_party_requests: 0,
            adapter_name: String::new(),
            adapter_version: String::new(),
        };

        info!("fetched {} bytes from {}", total_bytes, final_url);

        Ok(FetchResult {
            requested_url: url,
            final_url,
            content_type,
            content_type_raw,
            body: Bytes::from(body),
            audit,
            retrieved_at,
        })
    }
}

impl Default for NetworkGateway {
    fn default() -> Self {
        Self::new().expect("failed to build HTTP client")
    }
}
