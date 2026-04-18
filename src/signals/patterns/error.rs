//! Error pattern — burnt rust slow breathing. Spec §5.4: 2.5s cycle, 40–80%.

use crate::chroma::effects::{BreathingKind, EffectEnvelope};
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use crate::signals::palette::{self, BURNT_RUST};
use std::future::Future;
use std::pin::Pin;

pub struct ErrorPattern {
    sent: bool,
}

impl ErrorPattern {
    pub fn new() -> Self {
        Self { sent: false }
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
        session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let send = !self.sent;
        self.sent = true;
        Box::pin(async move {
            if send {
                let color = palette::scale(BURNT_RUST, 0.60);
                let effect = EffectEnvelope::breathing(BreathingKind::OneColor(color));
                session.put_effect("keyboard", &effect).await?;
            }
            Ok(true)
        })
    }
}
