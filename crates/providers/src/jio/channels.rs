use omega_core::{Channel, OmegaError, OmegaResult, ProviderKind};
use reqwest::Client;

use crate::jio::maps::{map_category, map_lang, value_to_bool, value_to_i64, value_to_string};
use crate::jio::models::{JioChannelListResponse, JioRawChannel};

const JIO_CHANNEL_API: &str = "https://jiotv.data.cdn.jio.com/apis/v3.0/getMobileChannelList/get/?os=android&devicetype=phone&usertype=tvYR7NSNn7rymo3F";
const JIO_LOGO_BASE: &str = "https://jiotvimages.cdn.jio.com/dare_images/images/";

pub async fn fetch_jio_channels(client: &Client) -> OmegaResult<Vec<Channel>> {
    let response = client
        .get(JIO_CHANNEL_API)
        .header(
            "user-agent",
            "Mozilla/5.0 (Linux; Android 13; Mobile) AppleWebKit/537.36 Omega-IPTV-Rust/1.0",
        )
        .header("accept", "application/json,text/plain,*/*")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OmegaError::provider(format!(
            "JioTV channel list failed with status {}",
            response.status()
        )));
    }

    let payload = response.json::<JioChannelListResponse>().await?;

    let mut channels = Vec::new();

    for raw in payload.result {
        if let Some(channel) = normalize_jio_channel(raw)? {
            channels.push(channel);
        }
    }

    Ok(channels)
}

pub fn normalize_jio_channel(raw: JioRawChannel) -> OmegaResult<Option<Channel>> {
    let Some(id) = value_to_string(&raw.channel_id) else {
        return Ok(None);
    };

    let Some(name) = raw.channel_name.clone() else {
        return Ok(None);
    };

    if name.trim().is_empty() {
        return Ok(None);
    }

    let language_id = value_to_i64(&raw.channel_language_id);
    let category_id = value_to_i64(&raw.channel_category_id);

    let is_hd = value_to_bool(&raw.is_hd);

    let mut channel = Channel::new(
        ProviderKind::Jio,
        id.clone(),
        name,
    );

    channel.language = Some(map_lang(language_id).to_string());
    channel.category = Some(map_category(category_id).to_string());
    channel.group = channel.category.clone();

    channel.quality = Some(if is_hd {
        "HD".to_string()
    } else {
        "SD".to_string()
    });

    channel.logo = raw.logo_url.as_ref().map(|logo| {
        if logo.starts_with("http") {
            logo.clone()
        } else {
            format!("{}{}", JIO_LOGO_BASE, logo)
        }
    });

    channel.premium = value_to_bool(&raw.is_premium);
    channel.catchup = value_to_bool(&raw.is_catchup_available);
    channel.hidden = value_to_bool(&raw.is_hidden);

    channel.raw = serde_json::to_value(&raw)?;

    Ok(Some(channel))
}
