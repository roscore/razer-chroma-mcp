//! Anthropic warm family — the only colors used by `signal_*` patterns. Spec §5.1.

use crate::chroma::color::Rgb;

pub const TERRACOTTA: Rgb = Rgb::new(0xD9, 0x77, 0x57);
pub const WARM_CREAM: Rgb = Rgb::new(0xF0, 0xEE, 0xE6);
pub const DEEP_EMBER: Rgb = Rgb::new(0x3E, 0x27, 0x23);
pub const BURNT_RUST: Rgb = Rgb::new(0x8D, 0x3A, 0x1C);

/// Linear interpolation between two RGB colors. `t` is clamped to [0, 1].
pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

/// Scale brightness. `factor` clamped to [0, 1].
pub fn scale(c: Rgb, factor: f32) -> Rgb {
    let f = factor.clamp(0.0, 1.0);
    let s = |x: u8| ((x as f32) * f).round() as u8;
    Rgb::new(s(c.r), s(c.g), s(c.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints_return_endpoints() {
        assert_eq!(lerp(TERRACOTTA, WARM_CREAM, 0.0), TERRACOTTA);
        assert_eq!(lerp(TERRACOTTA, WARM_CREAM, 1.0), WARM_CREAM);
    }

    #[test]
    fn scale_to_half_halves_channels() {
        let c = Rgb::new(100, 200, 50);
        assert_eq!(scale(c, 0.5), Rgb::new(50, 100, 25));
    }

    #[test]
    fn scale_is_clamped() {
        assert_eq!(scale(Rgb::new(100, 100, 100), 2.0), Rgb::new(100, 100, 100));
        assert_eq!(scale(Rgb::new(100, 100, 100), -1.0), Rgb::new(0, 0, 0));
    }
}
