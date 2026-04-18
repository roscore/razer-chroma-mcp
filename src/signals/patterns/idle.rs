//! Idle pattern — very slow deep ember breathing. Spec §5.4: 12s cycle, 5–12% brightness.
//! Uses the built-in `CHROMA_BREATHING` effect so we only PUT once.

use crate::chroma::effects::{BreathingKind, EffectEnvelope};
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use crate::signals::palette::{self, DEEP_EMBER};
use std::future::Future;
use std::pin::Pin;

pub struct IdlePattern {
    sent: bool,
}

impl IdlePattern {
    pub fn new() -> Self {
        Self { sent: false }
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
        session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let send = !self.sent;
        self.sent = true;
        Box::pin(async move {
            if send {
                let dim = palette::scale(DEEP_EMBER, 0.10); // center of 5–12% band
                let effect = EffectEnvelope::breathing(BreathingKind::OneColor(dim));
                session.put_effect("keyboard", &effect).await?;
            }
            Ok(true)
        })
    }
}
