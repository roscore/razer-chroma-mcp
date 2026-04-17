//! Chroma-supported device kinds and their grid dimensions.
//!
//! v0.1.0 patterns target the generic keyboard grid only (spec §5.6).
//! Other devices are addressable via low-level tools for basic colors.

use crate::error::{ChromaError, Result};

pub const KEYBOARD_ROWS: usize = 6;
pub const KEYBOARD_COLS: usize = 22;
pub const MOUSE_ROWS: usize = 9;
pub const MOUSE_COLS: usize = 7;
pub const MOUSEPAD_LEDS: usize = 20;
pub const HEADSET_LEDS: usize = 5;
pub const KEYPAD_ROWS: usize = 4;
pub const KEYPAD_COLS: usize = 5;
pub const CHROMALINK_LEDS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keyboard,
    Mouse,
    Mousepad,
    Headset,
    Keypad,
    ChromaLink,
}

impl Device {
    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Mousepad => "mousepad",
            Self::Headset => "headset",
            Self::Keypad => "keypad",
            Self::ChromaLink => "chromalink",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "keyboard" => Ok(Self::Keyboard),
            "mouse" => Ok(Self::Mouse),
            "mousepad" => Ok(Self::Mousepad),
            "headset" => Ok(Self::Headset),
            "keypad" => Ok(Self::Keypad),
            "chromalink" => Ok(Self::ChromaLink),
            other => Err(ChromaError::UnknownDevice(other.to_string())),
        }
    }

    /// Grid dimensions as `Some((rows, cols))` for grid devices, `None` for LED strips.
    pub const fn grid(self) -> Option<(usize, usize)> {
        match self {
            Self::Keyboard => Some((KEYBOARD_ROWS, KEYBOARD_COLS)),
            Self::Mouse => Some((MOUSE_ROWS, MOUSE_COLS)),
            Self::Keypad => Some((KEYPAD_ROWS, KEYPAD_COLS)),
            Self::Mousepad | Self::Headset | Self::ChromaLink => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_grid_is_6x22() {
        assert_eq!(Device::Keyboard.grid(), Some((6, 22)));
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(Device::parse("KEYBOARD").unwrap(), Device::Keyboard);
    }

    #[test]
    fn parse_rejects_unknown() {
        assert!(matches!(Device::parse("gpu"), Err(ChromaError::UnknownDevice(_))));
    }

    #[test]
    fn path_segment_stable() {
        assert_eq!(Device::Keyboard.path_segment(), "keyboard");
    }
}
