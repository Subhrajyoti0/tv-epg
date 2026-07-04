use crate::sqlite::DbPool;

pub struct ExportRepository {
    pool: DbPool,
}

impl ExportRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn unified_channels_json(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, UnifiedChannelExportRow>(
            r#"
            SELECT
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
                confidence_source
            FROM channels
            ORDER BY canonical_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let output = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "canonical_name": row.canonical_name,
                    "display_name": row.display_name,
                    "tvg_id": row.tvg_id,
                    "language": row.language,
                    "country": row.country,
                    "group": row.group_name,
                    "category": row.category,
                    "quality": row.quality,
                    "logo": row.logo,
                    "url": row.stream_url,
                    "confidence": row.confidence,
                    "confidence_source": row.confidence_source
                })
            })
            .collect();

        Ok(output)
    }

    pub async fn reviews_json(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, ReviewExportRow>(
            r#"
            SELECT
                id,
                source_provider,
                source_channel_id,
                source_name,
                best_score,
                reason,
                candidates_json,
                resolved,
                resolution,
                created_at,
                updated_at
            FROM reviews
            ORDER BY best_score DESC, source_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let output = rows
            .into_iter()
            .map(|row| {
                let candidates = row
                    .candidates_json
                    .as_deref()
                    .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
                    .unwrap_or_else(|| serde_json::json!([]));

                serde_json::json!({
                    "id": row.id,
                    "source_provider": row.source_provider,
                    "source_channel_id": row.source_channel_id,
                    "source_name": row.source_name,
                    "best_score": row.best_score,
                    "reason": row.reason,
                    "candidates": candidates,
                    "resolved": row.resolved != 0,
                    "resolution": row.resolution,
                    "created_at": row.created_at,
                    "updated_at": row.updated_at
                })
            })
            .collect();

        Ok(output)
    }

    pub async fn matches_json(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, MatchExportRow>(
            r#"
            SELECT
                source_provider,
                source_channel_id,
                target_provider,
                target_channel_id,
                unified_channel_id,
                score,
                decision,
                confidence,
                reason,
                created_at
            FROM matches
            ORDER BY score DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let output = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "source_provider": row.source_provider,
                    "source_channel_id": row.source_channel_id,
                    "target_provider": row.target_provider,
                    "target_channel_id": row.target_channel_id,
                    "unified_channel_id": row.unified_channel_id,
                    "score": row.score,
                    "decision": row.decision,
                    "confidence": row.confidence,
                    "reason": row.reason,
                    "created_at": row.created_at
                })
            })
            .collect();

        Ok(output)
    }
    
    pub async fn xmltv_channels_json(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, XmltvChannelRow>(
            r#"
            SELECT
                id,
                display_name,
                tvg_id,
                logo
            FROM channels
            WHERE tvg_id IS NOT NULL
              AND stream_url IS NOT NULL
            ORDER BY display_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let output = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id,
                    "display_name": row.display_name,
                    "tvg_id": row.tvg_id,
                    "logo": row.logo
                })
            })
            .collect();

        Ok(output)
    }

    pub async fn programme_channel_map_json(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let rows = sqlx::query_as::<_, ProgrammeChannelMapRow>(
            r#"
            SELECT
                m.source_channel_id,
                c.tvg_id
            FROM matches m
            JOIN channels c
              ON c.id = m.unified_channel_id
            WHERE m.source_provider = 'jio'
              AND m.decision = 'matched'
              AND c.tvg_id IS NOT NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let output = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "source_channel_id": row.source_channel_id,
                    "tvg_id": row.tvg_id
                })
            })
            .collect();

        Ok(output)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct UnifiedChannelExportRow {
    id: String,
    canonical_name: String,
    display_name: String,
    tvg_id: Option<String>,
    language: Option<String>,
    country: Option<String>,
    group_name: Option<String>,
    category: Option<String>,
    quality: Option<String>,
    logo: Option<String>,
    stream_url: Option<String>,
    confidence: f64,
    confidence_source: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReviewExportRow {
    id: String,
    source_provider: String,
    source_channel_id: String,
    source_name: String,
    best_score: f64,
    reason: String,
    candidates_json: Option<String>,
    resolved: i64,
    resolution: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, sqlx::FromRow)]
struct MatchExportRow {
    source_provider: String,
    source_channel_id: String,
    target_provider: Option<String>,
    target_channel_id: Option<String>,
    unified_channel_id: Option<String>,
    score: f64,
    decision: String,
    confidence: String,
    reason: Option<String>,
    created_at: String,
}

#[derive(Debug, sqlx::FromRow)]
struct XmltvChannelRow {
    id: String,
    display_name: String,
    tvg_id: Option<String>,
    logo: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ProgrammeChannelMapRow {
    source_channel_id: String,
    tvg_id: Option<String>,
}
