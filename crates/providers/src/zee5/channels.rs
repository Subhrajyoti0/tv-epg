use std::time::Duration;

use omega_core::{Channel, OmegaError, OmegaResult, ProviderKind};
use reqwest::Client;
use tokio::time::sleep;

use crate::zee5::logos::resolve_logo_from_image;
use crate::zee5::models::{Zee5CatalogItem, Zee5CatalogResponse};

const ZEE5_CATALOG_URL: &str = "https://catalogapi.zee5.com/v1/channel";
const PAGE_SIZE: usize = 25;

pub async fn fetch_zee5_channels(client: &Client) -> OmegaResult<Vec<Channel>> {
    let mut page = 1usize;
    let mut total = usize::MAX;
    let mut channels = Vec::new();

    while (page - 1) * PAGE_SIZE < total {
        let response = client
            .get(ZEE5_CATALOG_URL)
            .query(&[
                ("page", page.to_string()),
                ("page_size", PAGE_SIZE.to_string()),
            ])
            .header("accept", "application/json")
            .header("user-agent", "Mozilla/5.0 Omega-IPTV-Rust/1.0")
            .header("origin", "https://www.zee5.com")
            .header("referer", "https://www.zee5.com/")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OmegaError::provider(format!(
                "Zee5 catalog failed with status {}",
                response.status()
            )));
        }

        let payload = response.json::<Zee5CatalogResponse>().await?;

        if payload.items.is_empty() {
            break;
        }

        if let Some(api_total) = payload.total {
            total = api_total;
        }

        for item in payload.items {
            channels.push(normalize_zee5_channel(item)?);
        }

        page += 1;

        sleep(Duration::from_millis(1500)).await;
    }

    Ok(channels)
}

pub fn normalize_zee5_channel(item: Zee5CatalogItem) -> OmegaResult<Channel> {
    let name = item
        .title
        .clone()
        .or(item.original_title.clone())
        .unwrap_or_else(|| item.id.clone());

    let mut channel = Channel::new(
        ProviderKind::Zee5,
        item.id.clone(),
        name,
    );

    channel.language = item.languages.first().cloned();
    channel.logo = resolve_logo_from_image(&item.image);
    channel.group = Some("ZEE5".to_string());
    channel.category = Some("Entertainment".to_string());
    channel.raw = serde_json::to_value(&item)?;

    Ok(channel)
}
