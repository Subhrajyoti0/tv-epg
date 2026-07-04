use chrono::{DateTime, Utc};
use omega_core::{Channel, OmegaError, OmegaResult, Programme, ProviderKind};
use reqwest::Client;

use crate::zee5::models::Zee5EpgResponse;

const ZEE5_EPG_URL: &str = "https://gwapi.zee5.com/v1/epg";

pub async fn fetch_zee5_epg_for_channel(
    client: &Client,
    channel: &Channel,
) -> OmegaResult<Vec<Programme>> {
    let response = client
        .get(ZEE5_EPG_URL)
        .query(&[
            ("channels", channel.id.as_str()),
            ("start", "0"),
            ("end", "5"),
            ("page_size", "550"),
            ("translation", "en"),
            ("country", "IN"),
            ("time_offset", "+05:30"),
        ])
        .header("accept", "application/json")
        .header("user-agent", "Mozilla/5.0 Omega-IPTV-Rust/1.0")
        .header("origin", "https://www.zee5.com")
        .header("referer", "https://www.zee5.com/")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OmegaError::provider(format!(
            "Zee5 EPG failed for channel {} with status {}",
            channel.id,
            response.status()
        )));
    }

    let payload = response.json::<Zee5EpgResponse>().await?;

    let Some(entry) = payload.items.first() else {
        return Ok(Vec::new());
    };

    let mut programmes = Vec::new();

    for raw in &entry.items {
        if let Some(programme) = normalize_zee5_programme(raw, channel)? {
            programmes.push(programme);
        }
    }

    programmes.sort_by_key(|programme| programme.start);

    Ok(programmes)
}

pub fn normalize_zee5_programme(
    raw: &serde_json::Value,
    channel: &Channel,
) -> OmegaResult<Option<Programme>> {
    let Some(start) = extract_datetime(raw, &["start_time", "start", "from"]) else {
        return Ok(None);
    };

    let Some(stop) = extract_datetime(raw, &["end_time", "end", "to"]) else {
        return Ok(None);
    };

    if stop <= start {
        return Ok(None);
    }

    let title = extract_string(raw, &["title", "name"])
        .unwrap_or_else(|| "Unknown Programme".to_string());

    if title.trim().is_empty() {
        return Ok(None);
    }

    let subtitle = extract_string(raw, &["episode_title"]);
    let description = extract_string(raw, &[
        "description",
        "long_description",
        "short_description",
    ]);

    let categories = extract_genres(raw, "genres");
    let tags = extract_string_array(raw, "tags");

    let mut genres = categories.clone();

    for tag in tags {
        if !genres.contains(&tag) {
            genres.push(tag);
        }
    }

    let actors = extract_string_array(raw, "cast");
    let directors = extract_string_array(raw, "directors");

    let image = raw
        .get("image")
        .and_then(|image| {
            image
                .get("list")
                .or_else(|| image.get("cover"))
                .or_else(|| image.get("thumbnail"))
        })
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());

    let programme = Programme {
        provider: ProviderKind::Zee5,
        channel_id: channel.id.clone(),
        programme_id: extract_string(raw, &["id"]),
        title,
        subtitle,
        description,
        start,
        stop,
        categories,
        genres,
        language: extract_string(raw, &["language"])
            .or_else(|| channel.language.clone()),
        image,
        actors,
        directors,
        rating_system: extract_string(raw, &["rating_system"]),
        rating_value: extract_string(raw, &["rating"]),
        is_repeat: extract_bool(raw, &["is_repeat"]),
        is_live: extract_bool(raw, &["is_live"]),
        catchup: false,
        raw: raw.clone(),
    };

    Ok(Some(programme))
}

fn extract_string(raw: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = raw.get(key).and_then(|value| value.as_str()) {
            if !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn extract_bool(raw: &serde_json::Value, keys: &[&str]) -> bool {
    for key in keys {
        if let Some(value) = raw.get(key).and_then(|value| value.as_bool()) {
            return value;
        }
    }

    false
}

fn extract_datetime(
    raw: &serde_json::Value,
    keys: &[&str],
) -> Option<DateTime<Utc>> {
    for key in keys {
        let Some(value) = raw.get(key) else {
            continue;
        };

        if let Some(text) = value.as_str() {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(text) {
                return Some(parsed.with_timezone(&Utc));
            }
        }
    }

    None
}

fn extract_string_array(raw: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(array) = raw.get(key).and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    array
        .iter()
        .filter_map(|item| {
            if let Some(text) = item.as_str() {
                return Some(text.to_string());
            }

            if let Some(text) = item.get("value").and_then(|value| value.as_str()) {
                return Some(text.to_string());
            }

            None
        })
        .filter(|value| !value.trim().is_empty())
        .collect()
}

fn extract_genres(raw: &serde_json::Value, key: &str) -> Vec<String> {
    extract_string_array(raw, key)
}
