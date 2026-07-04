use async_trait::async_trait;
use omega_core::{
    Channel,
    OmegaResult,
    Programme,
    Provider,
    ProviderKind,
};
use reqwest::Client;

use crate::jio::channels::fetch_jio_channels;
use crate::jio::epg::fetch_jio_epg_for_channel;

#[derive(Clone)]
pub struct JioProvider {
    client: Client,
    start_offset: i32,
    end_offset: i32,
}

impl JioProvider {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            start_offset: 0,
            end_offset: 5,
        }
    }

    pub fn with_offsets(client: Client, start_offset: i32, end_offset: i32) -> Self {
        Self {
            client,
            start_offset,
            end_offset,
        }
    }
}

#[async_trait]
impl Provider for JioProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Jio
    }

    async fn health_check(&self) -> OmegaResult<bool> {
        let response = self
            .client
            .get("https://jiotv.data.cdn.jio.com/apis/v3.0/getMobileChannelList/get/")
            .query(&[
                ("os", "android"),
                ("devicetype", "phone"),
                ("usertype", "tvYR7NSNn7rymo3F"),
            ])
            .header(
                "user-agent",
                "Mozilla/5.0 (Linux; Android 13; Mobile) AppleWebKit/537.36 Omega-IPTV-Rust/1.0",
            )
            .header("accept", "application/json,text/plain,*/*")
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    async fn fetch_channels(&self) -> OmegaResult<Vec<Channel>> {
        fetch_jio_channels(&self.client).await
    }

    async fn fetch_programmes(&self, channels: &[Channel]) -> OmegaResult<Vec<Programme>> {
        let mut all_programmes = Vec::new();

        for channel in channels {
            if channel.hidden {
                continue;
            }

            let programmes = fetch_jio_epg_for_channel(
                &self.client,
                channel,
                self.start_offset,
                self.end_offset,
            )
            .await?;

            all_programmes.extend(programmes);

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(all_programmes)
    }
}
