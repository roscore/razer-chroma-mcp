//! Tracks "time since last signal" and fires a callback when idle timeout elapses.
//! Spec §5.9: default 10 min; Thinking/NeedsInput excluded from auto-entry.

use crate::signals::ClaudeState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub const DEFAULT_TIMEOUT_MS: u64 = 10 * 60 * 1_000;

pub fn timeout_from_env() -> Duration {
    std::env::var("RAZER_CHROMA_IDLE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(DEFAULT_TIMEOUT_MS))
}

#[derive(Clone)]
pub struct Inactivity {
    last: Arc<Mutex<Instant>>,
    cancel: CancellationToken,
}

impl Inactivity {
    pub fn new() -> Self {
        Self {
            last: Arc::new(Mutex::new(Instant::now())),
            cancel: CancellationToken::new(),
        }
    }

    pub async fn bump(&self) {
        *self.last.lock().await = Instant::now();
    }

    pub async fn last(&self) -> Instant {
        *self.last.lock().await
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Returns true if `state` is eligible for auto-Idle.
    pub fn eligible(state: ClaudeState) -> bool {
        matches!(state, ClaudeState::UserControlled | ClaudeState::Error)
    }

    /// Spawn a watcher that calls `on_idle()` once when the timeout elapses AND
    /// the current state is eligible. `get_state` samples state without mutex contention.
    pub fn spawn_watcher<G, F, Fut>(self, timeout: Duration, get_state: G, on_idle: F) -> Self
    where
        G: Fn() -> ClaudeState + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let me = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = me.cancel.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
                let last = *me.last.lock().await;
                if Instant::now().duration_since(last) >= timeout && Self::eligible(get_state()) {
                    on_idle().await;
                    me.bump().await;
                }
            }
        });
        self
    }
}

impl Default for Inactivity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn bump_resets_elapsed() {
        let i = Inactivity::new();
        let before = i.last().await;
        tokio::time::advance(Duration::from_secs(2)).await;
        i.bump().await;
        let after = i.last().await;
        assert!(after > before);
    }

    #[test]
    fn eligibility_excludes_thinking_and_needs_input() {
        assert!(!Inactivity::eligible(ClaudeState::Thinking));
        assert!(!Inactivity::eligible(ClaudeState::NeedsInput));
        assert!(Inactivity::eligible(ClaudeState::UserControlled));
        assert!(Inactivity::eligible(ClaudeState::Error));
    }
}
