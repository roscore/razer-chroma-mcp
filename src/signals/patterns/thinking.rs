//! `Thinking` state pattern — stub. Real implementation in T4.6.

use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use std::future::Future;
use std::pin::Pin;

pub struct ThinkingPattern;

impl ThinkingPattern {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThinkingPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for ThinkingPattern {
    fn next_frame<'a>(
        &'a mut self,
        _session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}
