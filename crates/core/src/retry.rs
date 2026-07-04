use std::future::Future;
use std::time::Duration;

use tokio::time::sleep;

use crate::error::{OmegaError, OmegaResult};

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub attempts: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            attempts: 3,
            base_delay_ms: 500,
            max_delay_ms: 5_000,
        }
    }
}

pub async fn retry_async<T, F, Fut>(
    mut operation: F,
    policy: RetryPolicy,
) -> OmegaResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = OmegaResult<T>>,
{
    let mut last_error: Option<OmegaError> = None;

    for attempt in 0..policy.attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = Some(error);

                if attempt + 1 < policy.attempts {
                    let delay = policy
                        .base_delay_ms
                        .saturating_mul(2_u64.saturating_pow(attempt as u32))
                        .min(policy.max_delay_ms);

                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| OmegaError::Unknown("retry failed".to_string())))
}
