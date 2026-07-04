use serde::{Deserialize, Serialize};

use crate::channel::Channel;
use crate::unified_channel::UnifiedChannel;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
    Review,
    Rejected,
}

impl Confidence {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.90 {
            Confidence::High
        } else if score >= 0.80 {
            Confidence::Medium
        } else if score >= 0.65 {
            Confidence::Low
        } else {
            Confidence::Review
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchDecision {
    Matched,
    NeedsReview,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResult {
    pub decision: MatchDecision,
    pub confidence: Confidence,
    pub score: f64,

    pub source: Channel,
    pub target: Option<Channel>,
    pub unified: Option<UnifiedChannel>,

    pub reason: Option<String>,
}

impl MatchResult {
    pub fn needs_review(source: Channel, reason: impl Into<String>) -> Self {
        Self {
            decision: MatchDecision::NeedsReview,
            confidence: Confidence::Review,
            score: 0.0,
            source,
            target: None,
            unified: None,
            reason: Some(reason.into()),
        }
    }
}
