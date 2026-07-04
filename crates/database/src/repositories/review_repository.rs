use omega_core::ReviewItem;

use crate::sqlite::DbPool;

pub struct ReviewRepository {
    pool: DbPool,
}

impl ReviewRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, item: &ReviewItem) -> anyhow::Result<()> {
        let candidates_json = serde_json::to_string(&item.candidates)?;

        sqlx::query(
            r#"
            INSERT INTO reviews (
                id,
                source_provider,
                source_channel_id,
                source_name,
                best_score,
                reason,
                candidates_json,
                resolved,
                resolution,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                best_score = excluded.best_score,
                reason = excluded.reason,
                candidates_json = excluded.candidates_json,
                resolved = excluded.resolved,
                resolution = excluded.resolution,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(item.id.to_string())
        .bind(item.source.provider.as_str())
        .bind(&item.source.id)
        .bind(&item.source.name)
        .bind(item.best_score)
        .bind(&item.reason)
        .bind(candidates_json)
        .bind(item.resolved as i32)
        .bind(&item.resolution)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn unresolved_count(&self) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM reviews WHERE resolved = 0")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }
}
