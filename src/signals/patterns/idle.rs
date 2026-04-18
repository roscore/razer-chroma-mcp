//! `Idle` state pattern — stub. Real implementation in T4.8.

use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use std::future::Future;
use std::pin::Pin;

pub struct IdlePattern;

impl IdlePattern {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IdlePattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for IdlePattern {
    fn next_frame<'a>(
        &'a mut self,
        _session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}
