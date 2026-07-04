use omega_core::{Channel, ProviderKind};

use crate::sqlite::DbPool;

pub struct ProviderChannelRepository {
    pool: DbPool,
}

impl ProviderChannelRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, channel: &Channel) -> anyhow::Result<()> {
        let raw_json = serde_json::to_string(&channel.raw)?;

        sqlx::query(
            r#"
            INSERT INTO provider_channels (
                provider_kind,
                provider_channel_id,
                name,
                language,
                country,
                group_name,
                category,
                quality,
                logo,
                stream_url,
                premium,
                catchup,
                hidden,
                raw_json,
                updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(provider_kind, provider_channel_id) DO UPDATE SET
                name = excluded.name,
                language = excluded.language,
                country = excluded.country,
                group_name = excluded.group_name,
                category = excluded.category,
                quality = excluded.quality,
                logo = excluded.logo,
                stream_url = excluded.stream_url,
                premium = excluded.premium,
                catchup = excluded.catchup,
                hidden = excluded.hidden,
                raw_json = excluded.raw_json,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(channel.provider.as_str())
        .bind(&channel.id)
        .bind(&channel.name)
        .bind(&channel.language)
        .bind(&channel.country)
        .bind(&channel.group)
        .bind(&channel.category)
        .bind(&channel.quality)
        .bind(&channel.logo)
        .bind(&channel.stream_url)
        .bind(channel.premium as i32)
        .bind(channel.catchup as i32)
        .bind(channel.hidden as i32)
        .bind(raw_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn count_by_provider(&self, provider_kind: &str) -> anyhow::Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM provider_channels WHERE provider_kind = ?")
                .bind(provider_kind)
                .fetch_one(&self.pool)
                .await?;

        Ok(row.0)
    }

    pub async fn list_by_provider(&self, provider_kind: &str) -> anyhow::Result<Vec<Channel>> {
        let rows = sqlx::query_as::<_, ProviderChannelRow>(
            r#"
            SELECT
                provider_kind,
                provider_channel_id,
                name,
                language,
                country,
                group_name,
                category,
                quality,
                logo,
                stream_url,
                premium,
                catchup,
                hidden,
                raw_json
            FROM provider_channels
            WHERE provider_kind = ?
            ORDER BY name ASC
            "#,
        )
        .bind(provider_kind)
        .fetch_all(&self.pool)
        .await?;

        let mut channels = Vec::new();

        for row in rows {
            let provider = provider_kind_from_str(&row.provider_kind);

            let raw = match row.raw_json {
                Some(text) => serde_json::from_str(&text).unwrap_or(serde_json::Value::Null),
                None => serde_json::Value::Null,
            };

            let mut channel = Channel::new(
                provider,
                row.provider_channel_id,
                row.name,
            );

            channel.language = row.language;
            channel.country = row.country;
            channel.group = row.group_name;
            channel.category = row.category;
            channel.quality = row.quality;
            channel.logo = row.logo;
            channel.stream_url = row.stream_url;
            channel.premium = row.premium != 0;
            channel.catchup = row.catchup != 0;
            channel.hidden = row.hidden != 0;
            channel.raw = raw;

            if provider == ProviderKind::IptvOrg {
                channel.tvg_id = Some(channel.id.clone());
                channel.tvg_name = Some(channel.name.clone());
            }

            channels.push(channel);
        }

        Ok(channels)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ProviderChannelRow {
    provider_kind: String,
    provider_channel_id: String,
    name: String,
    language: Option<String>,
    country: Option<String>,
    group_name: Option<String>,
    category: Option<String>,
    quality: Option<String>,
    logo: Option<String>,
    stream_url: Option<String>,
    premium: i64,
    catchup: i64,
    hidden: i64,
    raw_json: Option<String>,
}

fn provider_kind_from_str(value: &str) -> ProviderKind {
    match value {
        "zee5" => ProviderKind::Zee5,
        "jio" => ProviderKind::Jio,
        "iptv_org" => ProviderKind::IptvOrg,
        "sonyliv" => ProviderKind::SonyLiv,
        _ => ProviderKind::Unknown,
    }
}
