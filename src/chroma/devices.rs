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

/// Environment variable selecting which devices receive signal effects (issue #4).
pub const DEVICES_ENV: &str = "RAZER_CHROMA_DEVICES";

/// Devices used when [`DEVICES_ENV`] is unset or blank: keyboard only (v0.1.0 behavior).
pub const DEFAULT_DEVICES: &[Device] = &[Device::Keyboard];

/// Parse a comma-separated device list such as `"keyboard,mouse,mousepad"`.
///
/// Entries are trimmed and matched case-insensitively. Empty segments are
/// skipped, duplicates keep their first position, and any unknown name yields
/// [`ChromaError::UnknownDevice`]. An input with no entries returns an empty list.
pub fn parse_device_list(s: &str) -> Result<Vec<Device>> {
    let mut out = Vec::new();
    for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
        let dev = Device::parse(part)?;
        if !out.contains(&dev) {
            out.push(dev);
        }
    }
    Ok(out)
}

/// Resolve the enabled device list from an optional raw env value.
///
/// `None` or a value with no entries falls back to [`DEFAULT_DEVICES`].
pub fn enabled_devices_from(raw: Option<&str>) -> Result<Vec<Device>> {
    let list = match raw {
        Some(s) => parse_device_list(s)?,
        None => Vec::new(),
    };
    if list.is_empty() {
        Ok(DEFAULT_DEVICES.to_vec())
    } else {
        Ok(list)
    }
}

/// Read [`DEVICES_ENV`] from the process environment and resolve it.
pub fn enabled_devices() -> Result<Vec<Device>> {
    enabled_devices_from(std::env::var(DEVICES_ENV).ok().as_deref())
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
        assert!(matches!(
            Device::parse("gpu"),
            Err(ChromaError::UnknownDevice(_))
        ));
    }

    #[test]
    fn path_segment_stable() {
        assert_eq!(Device::Keyboard.path_segment(), "keyboard");
    }

    #[test]
    fn device_list_unset_defaults_to_keyboard() {
        assert_eq!(enabled_devices_from(None).unwrap(), vec![Device::Keyboard]);
    }

    #[test]
    fn device_list_blank_defaults_to_keyboard() {
        assert_eq!(
            enabled_devices_from(Some("")).unwrap(),
            vec![Device::Keyboard]
        );
        assert_eq!(
            enabled_devices_from(Some("  , ,")).unwrap(),
            vec![Device::Keyboard]
        );
    }

    #[test]
    fn device_list_preserves_order() {
        assert_eq!(
            parse_device_list("mousepad,keyboard,mouse").unwrap(),
            vec![Device::Mousepad, Device::Keyboard, Device::Mouse]
        );
    }

    #[test]
    fn device_list_trims_and_ignores_case() {
        assert_eq!(
            parse_device_list(" Keyboard , MOUSE ").unwrap(),
            vec![Device::Keyboard, Device::Mouse]
        );
    }

    #[test]
    fn device_list_dedups_keeping_first() {
        assert_eq!(
            parse_device_list("mouse,keyboard,Mouse,keyboard").unwrap(),
            vec![Device::Mouse, Device::Keyboard]
        );
    }

    #[test]
    fn device_list_skips_empty_segments() {
        assert_eq!(
            parse_device_list("keyboard,,mouse,").unwrap(),
            vec![Device::Keyboard, Device::Mouse]
        );
    }

    #[test]
    fn device_list_rejects_unknown() {
        match parse_device_list("keyboard,gpu") {
            Err(ChromaError::UnknownDevice(name)) => assert_eq!(name, "gpu"),
            other => panic!("expected UnknownDevice, got {other:?}"),
        }
    }

    #[test]
    fn device_list_does_not_mask_unknown_with_default() {
        assert!(enabled_devices_from(Some("gpu")).is_err());
    }
}
