use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::provider::ProviderKind;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Programme {
    pub provider: ProviderKind,

    pub channel_id: String,
    pub programme_id: Option<String>,

    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,

    pub start: DateTime<Utc>,
    pub stop: DateTime<Utc>,

    pub categories: Vec<String>,
    pub genres: Vec<String>,

    pub language: Option<String>,

    pub image: Option<String>,

    pub actors: Vec<String>,
    pub directors: Vec<String>,

    pub rating_system: Option<String>,
    pub rating_value: Option<String>,

    pub is_repeat: bool,
    pub is_live: bool,
    pub catchup: bool,

    pub raw: serde_json::Value,
}

impl Programme {
    pub fn duration_seconds(&self) -> i64 {
        (self.stop - self.start).num_seconds()
    }

    pub fn is_valid(&self) -> bool {
        !self.channel_id.is_empty()
            && !self.title.is_empty()
            && self.stop > self.start
    }
}
