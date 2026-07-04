use omega_core::{MatchDecision, MatchResult};

use crate::sqlite::DbPool;

pub struct MatchRepository {
    pool: DbPool,
}

impl MatchRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, result: &MatchResult) -> anyhow::Result<()> {
        let target_provider = result
            .target
            .as_ref()
            .map(|channel| channel.provider.as_str().to_string());

        let target_channel_id = result
            .target
            .as_ref()
            .map(|channel| channel.id.clone());

        let unified_channel_id = result
            .unified
            .as_ref()
            .map(|channel| channel.id.to_string());

        sqlx::query(
            r#"
            INSERT INTO matches (
                source_provider,
                source_channel_id,
                target_provider,
                target_channel_id,
                unified_channel_id,
                score,
                decision,
                confidence,
                reason
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(result.source.provider.as_str())
        .bind(&result.source.id)
        .bind(target_provider)
        .bind(target_channel_id)
        .bind(unified_channel_id)
        .bind(result.score)
        .bind(decision_to_str(&result.decision))
        .bind(format!("{:?}", result.confidence))
        .bind(&result.reason)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

fn decision_to_str(decision: &MatchDecision) -> &'static str {
    match decision {
        MatchDecision::Matched => "matched",
        MatchDecision::NeedsReview => "review",
        MatchDecision::Rejected => "rejected",
    }
}
