//! `NeedsInput` state pattern — stub. Real implementation in T4.7.

use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use std::future::Future;
use std::pin::Pin;

pub struct NeedsInputPattern;

impl NeedsInputPattern {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NeedsInputPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for NeedsInputPattern {
    fn next_frame<'a>(
        &'a mut self,
        _session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}
