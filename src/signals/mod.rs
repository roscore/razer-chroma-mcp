//! Claude state machine, animators, and pattern generators. See spec §5.

pub mod controller;
pub mod inactivity;
pub mod palette;
pub mod patterns;

/// The states that `Complete` can restore to. Spec §5.2 — kept as a separate
/// enum so that the type system enforces "Complete is never nested".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistentState {
    Off,
    UserControlled,
    Idle,
    Thinking,
    NeedsInput,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaudeState {
    Off,
    UserControlled,
    Idle,
    Thinking,
    NeedsInput,
    /// One-shot. On completion, controller restores `previous`.
    Complete { previous: PersistentState },
    Error,
}

impl ClaudeState {
    /// If this state is persistent (i.e., not `Complete`), return its `PersistentState`.
    pub fn as_persistent(self) -> Option<PersistentState> {
        match self {
            Self::Off => Some(PersistentState::Off),
            Self::UserControlled => Some(PersistentState::UserControlled),
            Self::Idle => Some(PersistentState::Idle),
            Self::Thinking => Some(PersistentState::Thinking),
            Self::NeedsInput => Some(PersistentState::NeedsInput),
            Self::Error => Some(PersistentState::Error),
            Self::Complete { .. } => None,
        }
    }
}

impl From<PersistentState> for ClaudeState {
    fn from(p: PersistentState) -> Self {
        match p {
            PersistentState::Off => Self::Off,
            PersistentState::UserControlled => Self::UserControlled,
            PersistentState::Idle => Self::Idle,
            PersistentState::Thinking => Self::Thinking,
            PersistentState::NeedsInput => Self::NeedsInput,
            PersistentState::Error => Self::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claudestate_is_copy_and_small() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<ClaudeState>();
        // Single-byte discriminants.
        assert!(std::mem::size_of::<ClaudeState>() <= 4);
    }

    #[test]
    fn as_persistent_returns_none_for_complete() {
        let s = ClaudeState::Complete { previous: PersistentState::Thinking };
        assert!(s.as_persistent().is_none());
    }

    #[test]
    fn persistent_round_trip() {
        for p in [
            PersistentState::Off,
            PersistentState::UserControlled,
            PersistentState::Idle,
            PersistentState::Thinking,
            PersistentState::NeedsInput,
            PersistentState::Error,
        ] {
            assert_eq!(ClaudeState::from(p).as_persistent(), Some(p));
        }
    }
}
