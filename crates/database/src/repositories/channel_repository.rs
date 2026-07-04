use omega_core::UnifiedChannel;

use crate::sqlite::DbPool;

pub struct ChannelRepository {
    pool: DbPool,
}

impl ChannelRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, channel: &UnifiedChannel) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO channels (
                id,
                canonical_name,
                display_name,
                tvg_id,
                language,
                country,
                group_name,
                category,
                quality,
                logo,
                stream_url,
                confidence,
                confidence_source,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                canonical_name = excluded.canonical_name,
                display_name = excluded.display_name,
                tvg_id = excluded.tvg_id,
                language = excluded.language,
                country = excluded.country,
                group_name = excluded.group_name,
                category = excluded.category,
                quality = excluded.quality,
                logo = excluded.logo,
                stream_url = excluded.stream_url,
                confidence = excluded.confidence,
                confidence_source = excluded.confidence_source,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(channel.id.to_string())
        .bind(&channel.canonical_name)
        .bind(&channel.display_name)
        .bind(&channel.tvg_id)
        .bind(&channel.language)
        .bind(&channel.country)
        .bind(&channel.group)
        .bind(&channel.category)
        .bind(&channel.quality)
        .bind(&channel.logo)
        .bind(&channel.stream_url)
        .bind(channel.confidence)
        .bind(&channel.confidence_source)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn count(&self) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM channels")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }
}
