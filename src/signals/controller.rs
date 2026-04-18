//! Central owner of the Chroma session and the ClaudeState machine.
//! The pure `next_state` function encodes spec §5.3 and is table-tested below.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig, SessionStatus};
use crate::error::Result;
use crate::signals::{ClaudeState, PersistentState};
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incoming {
    Thinking,
    NeedsInput,
    Idle,
    Complete,
    Error,
    LowLevelTool,
}

/// Pure state transition per spec §5.3. Returns the new state.
///
/// `Complete` auto-exits are handled separately (see `Controller::after_complete`)
/// rather than encoded here, because their trigger is "animator finished" rather
/// than an incoming signal.
pub fn next_state(from: ClaudeState, incoming: Incoming) -> ClaudeState {
    use ClaudeState as C;
    use Incoming as I;
    match (from, incoming) {
        // From Off
        (C::Off, I::Thinking) => C::Thinking,
        (C::Off, I::NeedsInput) => C::NeedsInput,
        (C::Off, I::Idle) => C::Idle,
        (C::Off, I::Complete) => C::Complete { previous: PersistentState::Off },
        (C::Off, I::Error) => C::Error,
        (C::Off, I::LowLevelTool) => C::UserControlled,

        // From UserControlled
        (C::UserControlled, I::Thinking) => C::Thinking,
        (C::UserControlled, I::NeedsInput) => C::NeedsInput,
        (C::UserControlled, I::Idle) => C::Idle,
        (C::UserControlled, I::Complete) => C::Complete { previous: PersistentState::UserControlled },
        (C::UserControlled, I::Error) => C::Error,
        (C::UserControlled, I::LowLevelTool) => C::UserControlled,

        // From Idle
        (C::Idle, I::Thinking) => C::Thinking,
        (C::Idle, I::NeedsInput) => C::NeedsInput,
        (C::Idle, I::Idle) => C::Idle,
        (C::Idle, I::Complete) => C::Complete { previous: PersistentState::Idle },
        (C::Idle, I::Error) => C::Error,
        (C::Idle, I::LowLevelTool) => C::UserControlled,

        // From Thinking
        (C::Thinking, I::Thinking) => C::Thinking, // restart animator
        (C::Thinking, I::NeedsInput) => C::NeedsInput,
        (C::Thinking, I::Idle) => C::Idle,
        (C::Thinking, I::Complete) => C::Complete { previous: PersistentState::Thinking },
        (C::Thinking, I::Error) => C::Error,
        (C::Thinking, I::LowLevelTool) => C::UserControlled,

        // From NeedsInput
        (C::NeedsInput, I::Thinking) => C::Thinking,
        (C::NeedsInput, I::NeedsInput) => C::NeedsInput, // restart animator
        (C::NeedsInput, I::Idle) => C::Idle,
        (C::NeedsInput, I::Complete) => C::Complete { previous: PersistentState::NeedsInput },
        (C::NeedsInput, I::Error) => C::Error,
        (C::NeedsInput, I::LowLevelTool) => C::UserControlled,

        // From Complete — interrupts the one-shot
        (C::Complete { .. }, I::Thinking) => C::Thinking,
        (C::Complete { .. }, I::NeedsInput) => C::NeedsInput,
        (C::Complete { .. }, I::Idle) => C::Idle,
        (C::Complete { previous }, I::Complete) => C::Complete { previous }, // restart
        (C::Complete { .. }, I::Error) => C::Error,
        (C::Complete { .. }, I::LowLevelTool) => C::UserControlled,

        // From Error
        (C::Error, I::Thinking) => C::Thinking,
        (C::Error, I::NeedsInput) => C::NeedsInput,
        (C::Error, I::Idle) => C::Idle,
        (C::Error, I::Complete) => C::Complete { previous: PersistentState::Error },
        (C::Error, I::Error) => C::Error, // restart
        (C::Error, I::LowLevelTool) => C::UserControlled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClaudeState as C;
    use Incoming as I;
    use PersistentState as P;

    macro_rules! cell {
        ($name:ident, $from:expr, $incoming:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(next_state($from, $incoming), $expected);
            }
        };
    }

    // Row: Off
    cell!(off_thinking,       C::Off, I::Thinking,     C::Thinking);
    cell!(off_needs_input,    C::Off, I::NeedsInput,   C::NeedsInput);
    cell!(off_idle,           C::Off, I::Idle,         C::Idle);
    cell!(off_complete,       C::Off, I::Complete,     C::Complete { previous: P::Off });
    cell!(off_error,          C::Off, I::Error,        C::Error);
    cell!(off_lowlevel,       C::Off, I::LowLevelTool, C::UserControlled);

    // Row: UserControlled
    cell!(uc_thinking,        C::UserControlled, I::Thinking,     C::Thinking);
    cell!(uc_needs_input,     C::UserControlled, I::NeedsInput,   C::NeedsInput);
    cell!(uc_idle,            C::UserControlled, I::Idle,         C::Idle);
    cell!(uc_complete,        C::UserControlled, I::Complete,     C::Complete { previous: P::UserControlled });
    cell!(uc_error,           C::UserControlled, I::Error,        C::Error);
    cell!(uc_lowlevel,        C::UserControlled, I::LowLevelTool, C::UserControlled);

    // Row: Idle
    cell!(idle_thinking,      C::Idle, I::Thinking,     C::Thinking);
    cell!(idle_needs_input,   C::Idle, I::NeedsInput,   C::NeedsInput);
    cell!(idle_idle,          C::Idle, I::Idle,         C::Idle);
    cell!(idle_complete,      C::Idle, I::Complete,     C::Complete { previous: P::Idle });
    cell!(idle_error,         C::Idle, I::Error,        C::Error);
    cell!(idle_lowlevel,      C::Idle, I::LowLevelTool, C::UserControlled);

    // Row: Thinking
    cell!(th_thinking,        C::Thinking, I::Thinking,     C::Thinking);
    cell!(th_needs_input,     C::Thinking, I::NeedsInput,   C::NeedsInput);
    cell!(th_idle,            C::Thinking, I::Idle,         C::Idle);
    cell!(th_complete,        C::Thinking, I::Complete,     C::Complete { previous: P::Thinking });
    cell!(th_error,           C::Thinking, I::Error,        C::Error);
    cell!(th_lowlevel,        C::Thinking, I::LowLevelTool, C::UserControlled);

    // Row: NeedsInput
    cell!(ni_thinking,        C::NeedsInput, I::Thinking,     C::Thinking);
    cell!(ni_needs_input,     C::NeedsInput, I::NeedsInput,   C::NeedsInput);
    cell!(ni_idle,            C::NeedsInput, I::Idle,         C::Idle);
    cell!(ni_complete,        C::NeedsInput, I::Complete,     C::Complete { previous: P::NeedsInput });
    cell!(ni_error,           C::NeedsInput, I::Error,        C::Error);
    cell!(ni_lowlevel,        C::NeedsInput, I::LowLevelTool, C::UserControlled);

    // Row: Complete — seeded with previous=Thinking
    cell!(cp_thinking,        C::Complete { previous: P::Thinking }, I::Thinking,     C::Thinking);
    cell!(cp_needs_input,     C::Complete { previous: P::Thinking }, I::NeedsInput,   C::NeedsInput);
    cell!(cp_idle,            C::Complete { previous: P::Thinking }, I::Idle,         C::Idle);
    cell!(cp_complete,        C::Complete { previous: P::Thinking }, I::Complete,     C::Complete { previous: P::Thinking });
    cell!(cp_error,           C::Complete { previous: P::Thinking }, I::Error,        C::Error);
    cell!(cp_lowlevel,        C::Complete { previous: P::Thinking }, I::LowLevelTool, C::UserControlled);

    // Row: Error
    cell!(err_thinking,       C::Error, I::Thinking,     C::Thinking);
    cell!(err_needs_input,    C::Error, I::NeedsInput,   C::NeedsInput);
    cell!(err_idle,           C::Error, I::Idle,         C::Idle);
    cell!(err_complete,       C::Error, I::Complete,     C::Complete { previous: P::Error });
    cell!(err_error,          C::Error, I::Error,        C::Error);
    cell!(err_lowlevel,       C::Error, I::LowLevelTool, C::UserControlled);
}

// ──────────────────────────────────────────────────────────────────────────────
// Controller (M3 surface retained; expanded in 4.5 to own the state machine).
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Controller {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
        }
    }
}

impl Controller {
    pub async fn session(&self) -> Result<Session> {
        let s = self
            .session
            .get_or_try_init(|| Session::init(self.client.clone(), SessionConfig::default()))
            .await?;
        Ok(s.clone())
    }

    pub async fn session_status(&self) -> SessionStatus {
        match self.session.get() {
            Some(s) => s.status().await,
            None => SessionStatus::Uninitialized,
        }
    }

    pub async fn set_static(&self, color: Rgb) -> Result<()> {
        let session = self.session().await?;
        let grid = keyboard_uniform(color);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
            .await
    }

    pub async fn clear_all(&self) -> Result<()> {
        if let Some(s) = self.session.get() {
            s.put_effect("keyboard", &EffectEnvelope::none()).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if let Some(s) = self.session.get() {
            s.shutdown().await;
        }
    }
}
