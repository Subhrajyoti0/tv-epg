use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
struct CircuitBreakerInner {
    state: CircuitState,
    failures: usize,
    opened_at: Option<Instant>,
}

#[derive(Clone)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerInner>>,
    failure_threshold: usize,
    reset_after: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: usize, reset_after: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerInner {
                state: CircuitState::Closed,
                failures: 0,
                opened_at: None,
            })),
            failure_threshold,
            reset_after,
        }
    }

    pub fn allow_request(&self) -> bool {
        let mut guard = self.inner.lock().expect("circuit breaker poisoned");

        match guard.state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open => {
                if let Some(opened_at) = guard.opened_at {
                    if opened_at.elapsed() >= self.reset_after {
                        guard.state = CircuitState::HalfOpen;
                        return true;
                    }
                }

                false
            }
        }
    }

    pub fn record_success(&self) {
        let mut guard = self.inner.lock().expect("circuit breaker poisoned");

        guard.state = CircuitState::Closed;
        guard.failures = 0;
        guard.opened_at = None;
    }

    pub fn record_failure(&self) {
        let mut guard = self.inner.lock().expect("circuit breaker poisoned");

        guard.failures += 1;

        if guard.failures >= self.failure_threshold {
            guard.state = CircuitState::Open;
            guard.opened_at = Some(Instant::now());
        }
    }

    pub fn state(&self) -> CircuitState {
        self.inner.lock().expect("circuit breaker poisoned").state
    }
}
