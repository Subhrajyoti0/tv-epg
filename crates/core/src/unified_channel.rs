use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::provider::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderReference {
    pub provider: ProviderKind,
    pub id: String,
    pub name: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedChannel {
    pub id: Uuid,

    pub canonical_name: String,
    pub display_name: String,

    pub tvg_id: Option<String>,

    pub aliases: Vec<String>,
    pub providers: Vec<ProviderReference>,

    pub language: Option<String>,
    pub country: Option<String>,
    pub group: Option<String>,
    pub category: Option<String>,
    pub quality: Option<String>,

    pub logo: Option<String>,
    pub stream_url: Option<String>,

    pub confidence: f64,
    pub confidence_source: String,
}

impl UnifiedChannel {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();

        Self {
            id: Uuid::new_v4(),
            canonical_name: name.clone(),
            display_name: name,
            tvg_id: None,
            aliases: Vec::new(),
            providers: Vec::new(),
            language: None,
            country: None,
            group: None,
            category: None,
            quality: None,
            logo: None,
            stream_url: None,
            confidence: 0.0,
            confidence_source: "unknown".to_string(),
        }
    }

    pub fn add_provider_reference(&mut self, reference: ProviderReference) {
        self.providers.push(reference);
    }
}
