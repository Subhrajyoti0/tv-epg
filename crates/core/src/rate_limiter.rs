use std::future::Future;
use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::error::{OmegaError, OmegaResult};

#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
        }
    }

    pub async fn run<T, F, Fut>(&self, task: F) -> OmegaResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = OmegaResult<T>>,
    {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| OmegaError::Unknown("rate limiter closed".to_string()))?;

        task().await
    }
}
