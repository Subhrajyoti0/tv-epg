use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::channel::Channel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub id: Uuid,

    pub created_at: DateTime<Utc>,

    pub source: Channel,
    pub candidates: Vec<Channel>,

    pub best_score: f64,
    pub reason: String,

    pub resolved: bool,
    pub resolution: Option<String>,
}

impl ReviewItem {
    pub fn new(source: Channel, reason: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            source,
            candidates: Vec::new(),
            best_score: 0.0,
            reason: reason.into(),
            resolved: false,
            resolution: None,
        }
    }
}
