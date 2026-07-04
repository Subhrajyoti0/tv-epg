use serde::{Deserialize, Serialize};

use crate::provider::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub provider: ProviderKind,

    pub id: String,
    pub name: String,

    pub tvg_id: Option<String>,
    pub tvg_name: Option<String>,

    pub language: Option<String>,
    pub country: Option<String>,
    pub group: Option<String>,
    pub category: Option<String>,

    pub quality: Option<String>,

    pub logo: Option<String>,
    pub stream_url: Option<String>,

    pub premium: bool,
    pub catchup: bool,
    pub hidden: bool,

    pub raw: serde_json::Value,
}

impl Channel {
    pub fn new(provider: ProviderKind, id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            provider,
            id: id.into(),
            name: name.into(),
            tvg_id: None,
            tvg_name: None,
            language: None,
            country: None,
            group: None,
            category: None,
            quality: None,
            logo: None,
            stream_url: None,
            premium: false,
            catchup: false,
            hidden: false,
            raw: serde_json::Value::Null,
        }
    }

    pub fn display_key(&self) -> String {
        format!("{}:{}", self.provider.as_str(), self.id)
    }
}
