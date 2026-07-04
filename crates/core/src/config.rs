use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmegaConfig {
    pub environment: String,
    pub database: DatabaseConfig,
    pub output: OutputConfig,
    pub http: HttpConfig,
    pub providers: ProviderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub directory: String,
    pub xmltv_file: String,
    pub playlist_file: String,
    pub review_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub timeout_seconds: u64,
    pub retries: usize,
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub zee5_enabled: bool,
    pub jio_enabled: bool,
    pub iptv_org_enabled: bool,
}

impl Default for OmegaConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            database: DatabaseConfig {
                url: "sqlite://omega.db".to_string(),
                max_connections: 10,
            },
            output: OutputConfig {
                directory: "output".to_string(),
                xmltv_file: "output/omega.xml".to_string(),
                playlist_file: "output/omega.m3u".to_string(),
                review_file: "output/review.json".to_string(),
            },
            http: HttpConfig {
                timeout_seconds: 20,
                retries: 3,
                user_agent: "Mozilla/5.0 Omega-IPTV-Rust/1.0".to_string(),
            },
            providers: ProviderConfig {
                zee5_enabled: true,
                jio_enabled: true,
                iptv_org_enabled: true,
            },
        }
    }
}
