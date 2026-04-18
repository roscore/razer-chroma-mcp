//! `Complete` state pattern — stub. Real implementation in T4.9.
//!
//! This is a one-shot pattern: `next_frame` eventually returns `Ok(false)` so
//! the animator notifies `finished`, which the controller uses to auto-transition
//! back to the previous persistent state. The stub finishes after a single tick
//! so the auto-restore path can be exercised before the real pattern lands.

use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use std::future::Future;
use std::pin::Pin;

pub struct CompletePattern;

impl CompletePattern {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CompletePattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for CompletePattern {
    fn next_frame<'a>(
        &'a mut self,
        _session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        // Finish immediately so Complete auto-restore is wired.
        Box::pin(async { Ok(false) })
    }
}
