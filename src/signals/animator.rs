//! Animator runtime: a cancellable tokio task that pushes frames at 20 FPS.
//! Per spec §5.7: graceful cancel waits ≤200ms; force cancel drops immediately.

#[cfg(test)]
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub const FPS: u64 = 20;
pub const TICK_MS: u64 = 1_000 / FPS;

pub type BoxedFrame<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// A pattern generator: given a session and a tick index (0-based, ticks at `TICK_MS` intervals),
/// emits one frame by calling `session.put_effect(...)`. Return `Ok(true)` to continue,
/// `Ok(false)` to signal the pattern has finished (one-shot patterns use this).
pub trait Pattern: Send + Sync + 'static {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>>;
}

pub struct AnimatorHandle {
    cancel: CancellationToken,
    join: JoinHandle<()>,
    /// Fires when the animator observes its pattern returning `Ok(false)`
    /// (one-shot complete). Multiple observers may clone this Arc.
    pub finished: Arc<tokio::sync::Notify>,
}

impl AnimatorHandle {
    /// Graceful cancel: signal then wait up to 200ms for the task to exit.
    /// If it doesn't exit in time, we force-drop. Spec §5.7.
    pub async fn cancel_graceful(self) {
        self.cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_millis(200), self.join).await;
    }

    /// Force cancel: drop the handle immediately without awaiting.
    pub fn cancel_force(self) {
        self.cancel.cancel();
        self.join.abort();
    }
}

pub fn spawn<P: Pattern>(session: Session, mut pattern: P) -> AnimatorHandle {
    let cancel = CancellationToken::new();
    let child_cancel = cancel.clone();
    let finished = Arc::new(tokio::sync::Notify::new());
    let finished_task = finished.clone();
    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(TICK_MS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut tick: u64 = 0;
        loop {
            tokio::select! {
                _ = child_cancel.cancelled() => break,
                _ = ticker.tick() => {
                    let res = pattern.next_frame(&session, tick).await;
                    match res {
                        Ok(true) => {}
                        Ok(false) => {
                            finished_task.notify_waiters();
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("animator frame error: {e}");
                        }
                    }
                    tick += 1;
                }
            }
        }
    });
    AnimatorHandle {
        cancel,
        join,
        finished,
    }
}

/// No-op animator — used when the state has no animation (e.g., `UserControlled`
/// after a user frame has been placed).
pub struct IdlePattern;
impl Pattern for IdlePattern {
    fn next_frame<'a>(
        &'a mut self,
        _s: &'a Session,
        _t: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}

// A one-shot helper used by unit tests.
#[cfg(test)]
pub struct OneShotPattern {
    pub total_frames: u64,
    pub last_effect: Arc<std::sync::Mutex<Option<EffectEnvelope>>>,
}

#[cfg(test)]
impl Pattern for OneShotPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let cont = tick + 1 < self.total_frames;
        let ef = EffectEnvelope::none();
        let slot = self.last_effect.clone();
        Box::pin(async move {
            *slot.lock().unwrap() = Some(ef.clone());
            session.put_effect("keyboard", &ef).await?;
            Ok(cont)
        })
    }
}
