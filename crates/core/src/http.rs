use std::time::Duration;

use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, HeaderMap, HeaderValue, PRAGMA, USER_AGENT};

use crate::error::OmegaResult;

#[derive(Debug, Clone)]
pub struct HttpClientFactory {
    timeout: Duration,
    user_agent: String,
}

impl HttpClientFactory {
    pub fn new(timeout_seconds: u64, user_agent: impl Into<String>) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_seconds),
            user_agent: user_agent.into(),
        }
    }

    pub fn build(&self) -> OmegaResult<reqwest::Client> {
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("Omega-IPTV-Rust")),
        );

        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json,text/plain,*/*"),
        );

        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-GB,en;q=0.9"),
        );

        headers.insert(
            CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        );

        headers.insert(
            PRAGMA,
            HeaderValue::from_static("no-cache"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(self.timeout)
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .pool_max_idle_per_host(16)
            .build()?;

        Ok(client)
    }
}
