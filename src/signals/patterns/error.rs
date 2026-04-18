//! `Error` state pattern — stub. Real implementation in T4.8.

use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use std::future::Future;
use std::pin::Pin;

pub struct ErrorPattern;

impl ErrorPattern {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ErrorPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for ErrorPattern {
    fn next_frame<'a>(
        &'a mut self,
        _session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}
