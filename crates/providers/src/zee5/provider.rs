use async_trait::async_trait;
use omega_core::{
    Channel,
    OmegaResult,
    Programme,
    Provider,
    ProviderKind,
};
use reqwest::Client;

use crate::zee5::channels::fetch_zee5_channels;
use crate::zee5::epg::fetch_zee5_epg_for_channel;

#[derive(Clone)]
pub struct Zee5Provider {
    client: Client,
}

impl Zee5Provider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Provider for Zee5Provider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Zee5
    }

    async fn health_check(&self) -> OmegaResult<bool> {
        let response = self
            .client
            .get("https://catalogapi.zee5.com/v1/channel")
            .query(&[
                ("page", "1"),
                ("page_size", "1"),
            ])
            .header("accept", "application/json")
            .header("user-agent", "Mozilla/5.0 Omega-IPTV-Rust/1.0")
            .header("origin", "https://www.zee5.com")
            .header("referer", "https://www.zee5.com/")
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn fetch_channels(&self) -> OmegaResult<Vec<Channel>> {
        fetch_zee5_channels(&self.client).await
    }

    async fn fetch_programmes(&self, channels: &[Channel]) -> OmegaResult<Vec<Programme>> {
        let mut all_programmes = Vec::new();

        for channel in channels {
            let programmes = fetch_zee5_epg_for_channel(&self.client, channel).await?;
            all_programmes.extend(programmes);

            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        }

        Ok(all_programmes)
    }
}
