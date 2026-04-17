//! RGB color type, hex parsing, and BGR u32 encoding for the Chroma SDK.

use crate::error::{ChromaError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0, 0, 0);

    /// Parse a hex color string like `#RRGGBB` or `#RGB`. Leading `#` is required.
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim();
        let body = s.strip_prefix('#')
            .ok_or_else(|| ChromaError::InvalidColor(format!("missing leading '#': {s}")))?;
        let parse_u8 = |slice: &str| u8::from_str_radix(slice, 16)
            .map_err(|_| ChromaError::InvalidColor(format!("not valid hex: {s}")));
        match body.len() {
            6 => Ok(Self::new(parse_u8(&body[0..2])?, parse_u8(&body[2..4])?, parse_u8(&body[4..6])?)),
            3 => {
                let expand = |c: char| u8::from_str_radix(&format!("{c}{c}"), 16)
                    .map_err(|_| ChromaError::InvalidColor(format!("not valid hex: {s}")));
                let mut chars = body.chars();
                Ok(Self::new(
                    expand(chars.next().unwrap())?,
                    expand(chars.next().unwrap())?,
                    expand(chars.next().unwrap())?,
                ))
            }
            _ => Err(ChromaError::InvalidColor(format!("length must be 3 or 6 hex digits: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_long_hex() {
        assert_eq!(Rgb::from_hex("#D97757").unwrap(), Rgb::new(0xD9, 0x77, 0x57));
    }

    #[test]
    fn parses_short_hex() {
        assert_eq!(Rgb::from_hex("#F0E").unwrap(), Rgb::new(0xFF, 0x00, 0xEE));
    }

    #[test]
    fn rejects_missing_hash() {
        assert!(matches!(Rgb::from_hex("D97757"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn rejects_bad_length() {
        assert!(matches!(Rgb::from_hex("#D977"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn rejects_non_hex_chars() {
        assert!(matches!(Rgb::from_hex("#GGGGGG"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn parses_case_insensitive() {
        assert_eq!(Rgb::from_hex("#d97757").unwrap(), Rgb::from_hex("#D97757").unwrap());
    }
}
