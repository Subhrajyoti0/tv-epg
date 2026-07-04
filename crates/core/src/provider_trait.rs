use async_trait::async_trait;

use crate::channel::Channel;
use crate::error::OmegaResult;
use crate::programme::Programme;
use crate::provider::ProviderKind;

#[async_trait]
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    fn name(&self) -> &'static str {
        self.kind().as_str()
    }

    async fn health_check(&self) -> OmegaResult<bool>;

    async fn fetch_channels(&self) -> OmegaResult<Vec<Channel>>;

    async fn fetch_programmes(&self, channels: &[Channel]) -> OmegaResult<Vec<Programme>>;
}
