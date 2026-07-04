use chrono::{DateTime, Utc};
use omega_core::{Programme, ProviderKind};

use crate::sqlite::DbPool;

pub struct ProgrammeRepository {
    pool: DbPool,
}

impl ProgrammeRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn insert_ignore(&self, programme: &Programme) -> anyhow::Result<()> {
        let categories_json = serde_json::to_string(&programme.categories)?;
        let genres_json = serde_json::to_string(&programme.genres)?;
        let actors_json = serde_json::to_string(&programme.actors)?;
        let directors_json = serde_json::to_string(&programme.directors)?;
        let raw_json = serde_json::to_string(&programme.raw)?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO programmes (
                provider_kind,
                channel_id,
                programme_id,
                title,
                subtitle,
                description,
                start_time,
                stop_time,
                categories_json,
                genres_json,
                language,
                image,
                actors_json,
                directors_json,
                rating_system,
                rating_value,
                is_repeat,
                is_live,
                catchup,
                raw_json
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(programme.provider.as_str())
        .bind(&programme.channel_id)
        .bind(&programme.programme_id)
        .bind(&programme.title)
        .bind(&programme.subtitle)
        .bind(&programme.description)
        .bind(programme.start.to_rfc3339())
        .bind(programme.stop.to_rfc3339())
        .bind(categories_json)
        .bind(genres_json)
        .bind(&programme.language)
        .bind(&programme.image)
        .bind(actors_json)
        .bind(directors_json)
        .bind(&programme.rating_system)
        .bind(&programme.rating_value)
        .bind(programme.is_repeat as i32)
        .bind(programme.is_live as i32)
        .bind(programme.catchup as i32)
        .bind(raw_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn count(&self) -> anyhow::Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM programmes")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.0)
    }

    pub async fn list_all(&self) -> anyhow::Result<Vec<Programme>> {
        let rows = sqlx::query_as::<_, ProgrammeRow>(
            r#"
            SELECT
                provider_kind,
                channel_id,
                programme_id,
                title,
                subtitle,
                description,
                start_time,
                stop_time,
                categories_json,
                genres_json,
                language,
                image,
                actors_json,
                directors_json,
                rating_system,
                rating_value,
                is_repeat,
                is_live,
                catchup,
                raw_json
            FROM programmes
            ORDER BY start_time ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut programmes = Vec::new();

        for row in rows {
            let start = DateTime::parse_from_rfc3339(&row.start_time)?.with_timezone(&Utc);
            let stop = DateTime::parse_from_rfc3339(&row.stop_time)?.with_timezone(&Utc);

            let categories = parse_json_vec(row.categories_json);
            let genres = parse_json_vec(row.genres_json);
            let actors = parse_json_vec(row.actors_json);
            let directors = parse_json_vec(row.directors_json);

            let raw = row
                .raw_json
                .as_deref()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(text).ok())
                .unwrap_or(serde_json::Value::Null);

            programmes.push(Programme {
                provider: provider_kind_from_str(&row.provider_kind),
                channel_id: row.channel_id,
                programme_id: row.programme_id,
                title: row.title,
                subtitle: row.subtitle,
                description: row.description,
                start,
                stop,
                categories,
                genres,
                language: row.language,
                image: row.image,
                actors,
                directors,
                rating_system: row.rating_system,
                rating_value: row.rating_value,
                is_repeat: row.is_repeat != 0,
                is_live: row.is_live != 0,
                catchup: row.catchup != 0,
                raw,
            });
        }

        Ok(programmes)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ProgrammeRow {
    provider_kind: String,
    channel_id: String,
    programme_id: Option<String>,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    start_time: String,
    stop_time: String,
    categories_json: Option<String>,
    genres_json: Option<String>,
    language: Option<String>,
    image: Option<String>,
    actors_json: Option<String>,
    directors_json: Option<String>,
    rating_system: Option<String>,
    rating_value: Option<String>,
    is_repeat: i64,
    is_live: i64,
    catchup: i64,
    raw_json: Option<String>,
}

fn parse_json_vec(value: Option<String>) -> Vec<String> {
    value
        .and_then(|text| serde_json::from_str::<Vec<String>>(&text).ok())
        .unwrap_or_default()
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
