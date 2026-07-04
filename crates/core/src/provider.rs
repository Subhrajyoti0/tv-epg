use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderKind {
    Zee5,
    Jio,
    IptvOrg,
    SonyLiv,
    Unknown,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Zee5 => "zee5",
            ProviderKind::Jio => "jio",
            ProviderKind::IptvOrg => "iptv_org",
            ProviderKind::SonyLiv => "sonyliv",
            ProviderKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub kind: ProviderKind,
    pub name: String,
    pub enabled: bool,
    pub base_url: Option<String>,
    pub priority: u8,
}

impl ProviderInfo {
    pub fn new(kind: ProviderKind, name: impl Into<String>) -> Self {
        Self {
            kind,
            name: name.into(),
            enabled: true,
            base_url: None,
            priority: 100,
        }
    }
}
