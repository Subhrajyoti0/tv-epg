use chrono::{DateTime, TimeZone, Utc};
use omega_core::{Channel, OmegaError, OmegaResult, Programme, ProviderKind};
use reqwest::Client;

use crate::jio::models::JioEpgResponse;

const JIO_EPG_API: &str = "https://jiotv.data.cdn.jio.com/apis/v1.3/getepg/get";
const JIO_POSTER_BASE: &str = "https://jiotv.catchup.cdn.jio.com/dare_images/shows/";

pub async fn fetch_jio_epg(
    client: &Client,
    channel_id: &str,
    offset: i32,
    lang_id: i64,
) -> OmegaResult<Vec<serde_json::Value>> {
    let response = client
        .get(JIO_EPG_API)
        .query(&[
            ("channel_id", channel_id.to_string()),
            ("offset", offset.to_string()),
            ("langId", lang_id.to_string()),
        ])
        .header("user-agent", "Mozilla/5.0 Omega-IPTV-Rust/1.0")
        .header("accept", "application/json")
        .send()
        .await?;

    if response.status().as_u16() == 404 {
        return Ok(Vec::new());
    }

    if !response.status().is_success() {
        return Err(OmegaError::provider(format!(
            "JioTV EPG failed for channel {} offset {} with status {}",
            channel_id,
            offset,
            response.status()
        )));
    }

    let payload = response.json::<JioEpgResponse>().await?;

    if let Some(epg) = payload.epg {
        return Ok(epg);
    }

    if let Some(result) = payload.result {
        return Ok(result);
    }

    Ok(Vec::new())
}

pub async fn fetch_jio_epg_for_channel(
    client: &Client,
    channel: &Channel,
    start_offset: i32,
    end_offset: i32,
) -> OmegaResult<Vec<Programme>> {
    let lang_id = channel
        .raw
        .get("channel_language_id")
        .and_then(|value| value.as_i64())
        .unwrap_or(6);

    let mut programmes = Vec::new();

    for offset in start_offset..=end_offset {
        let raw_items = fetch_jio_epg(client, &channel.id, offset, lang_id).await?;

        for raw in raw_items {
            if let Some(programme) = normalize_jio_programme(&raw, channel, offset)? {
                programmes.push(programme);
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    programmes.sort_by_key(|programme| programme.start);
    programmes.dedup_by(|a, b| {
        a.channel_id == b.channel_id
            && a.start == b.start
            && a.stop == b.stop
            && a.title == b.title
    });

    Ok(programmes)
}

pub fn normalize_jio_programme(
    raw: &serde_json::Value,
    channel: &Channel,
    _offset: i32,
) -> OmegaResult<Option<Programme>> {
    let Some(start) = extract_epoch_datetime(raw, &["startEpoch", "serverEpoch"]) else {
        return Ok(None);
    };

    let Some(stop) = extract_epoch_datetime(raw, &["endEpoch"]) else {
        return Ok(None);
    };

    if stop <= start {
        return Ok(None);
    }

    let title = extract_string(raw, &["showname", "showName", "title"])
        .unwrap_or_else(|| "Unknown Programme".to_string());

    if title.trim().is_empty() {
        return Ok(None);
    }

    let subtitle = raw
        .get("episode_num")
        .and_then(|value| {
            if value.is_null() {
                None
            } else {
                Some(format!("Episode {}", value))
            }
        });

    let description = extract_string(raw, &["episode_desc", "description"]);

    let category = extract_string(raw, &["showCategory"])
        .or_else(|| channel.category.clone());

    let mut categories = Vec::new();

    if let Some(category) = category {
        categories.push(category);
    }

    let genres = extract_string_array(raw, "showGenre");

    for genre in &genres {
        if !categories.contains(genre) {
            categories.push(genre.clone());
        }
    }

    let image = extract_image(raw);

    let actors = extract_people(raw, "starCast");
    let directors = extract_people(raw, "director");

    let programme = Programme {
        provider: ProviderKind::Jio,
        channel_id: raw
            .get("channel_id")
            .and_then(|value| {
                if let Some(text) = value.as_str() {
                    Some(text.to_string())
                } else {
                    value.as_i64().map(|number| number.to_string())
                }
            })
            .unwrap_or_else(|| channel.id.clone()),
        programme_id: extract_string(raw, &["showId"]),
        title,
        subtitle,
        description,
        start,
        stop,
        categories,
        genres,
        language: channel.language.clone(),
        image,
        actors,
        directors,
        rating_system: Some("JioTV".to_string()),
        rating_value: extract_string(raw, &["pcr"]),
        is_repeat: extract_bool(raw, &["willRepeat", "isRepeat"]),
        is_live: extract_bool(raw, &["isLiveAvailable"]),
        catchup: extract_bool(raw, &["isCatchupAvailable"]),
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

fn extract_epoch_datetime(
    raw: &serde_json::Value,
    keys: &[&str],
) -> Option<DateTime<Utc>> {
    for key in keys {
        let Some(value) = raw.get(key) else {
            continue;
        };

        let epoch = if let Some(number) = value.as_i64() {
            number
        } else if let Some(text) = value.as_str() {
            text.parse::<i64>().ok()?
        } else {
            continue;
        };

        let millis = if epoch > 9_999_999_999 {
            epoch
        } else {
            epoch * 1000
        };

        return Utc.timestamp_millis_opt(millis).single();
    }

    None
}

fn extract_string_array(raw: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(array) = raw.get(key).and_then(|value| value.as_array()) else {
        return Vec::new();
    };

    array
        .iter()
        .filter_map(|item| item.as_str().map(|text| text.to_string()))
        .filter(|text| !text.trim().is_empty())
        .collect()
}

fn extract_people(raw: &serde_json::Value, key: &str) -> Vec<String> {
    let Some(value) = raw.get(key) else {
        return Vec::new();
    };

    if let Some(array) = value.as_array() {
        return array
            .iter()
            .filter_map(|item| item.as_str().map(|text| text.trim().to_string()))
            .filter(|text| !text.is_empty())
            .collect();
    }

    if let Some(text) = value.as_str() {
        return text
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
    }

    Vec::new()
}

fn extract_image(raw: &serde_json::Value) -> Option<String> {
    let image = raw
        .get("assets")
        .and_then(|assets| assets.get("16:9"))
        .and_then(|ratio| {
            ratio
                .get("episode")
                .or_else(|| ratio.get("program"))
        })
        .and_then(|value| value.as_str())
        .or_else(|| raw.get("episodeThumbnail").and_then(|value| value.as_str()))
        .or_else(|| raw.get("episodePoster").and_then(|value| value.as_str()))?;

    if image.starts_with("http") {
        Some(image.to_string())
    } else {
        Some(format!("{}{}", JIO_POSTER_BASE, image))
    }
}
