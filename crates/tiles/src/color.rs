use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn hex(rgb: u32) -> Self {
        let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
        let b = (rgb & 0xFF) as f32 / 255.0;
        Self {
            r: srgb_to_linear(r),
            g: srgb_to_linear(g),
            b: srgb_to_linear(b),
            a: 1.0,
        }
    }

    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: 1.0,
        }
    }

    pub fn alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub(crate) fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

pub(crate) fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn lighter(base: Color) -> Color {
    mix(base, Color::linear(1.0, 1.0, 1.0, 1.0), 0.45)
}

fn light(base: Color) -> Color {
    mix(base, Color::linear(1.0, 1.0, 1.0, 1.0), 0.25)
}

fn dark(base: Color) -> Color {
    mix(base, Color::linear(0.0, 0.0, 0.0, 1.0), 0.25)
}

fn darker(base: Color) -> Color {
    mix(base, Color::linear(0.0, 0.0, 0.0, 1.0), 0.45)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::linear(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn scale(base: Color) -> ColorScale {
    ColorScale {
        darker: darker(base),
        dark: dark(base),
        base,
        light: light(base),
        lighter: lighter(base),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ColorScale {
    pub darker: Color,
    pub dark: Color,
    pub base: Color,
    pub light: Color,
    pub lighter: Color,
}

#[derive(Clone, Debug)]
pub struct Theme {
    // Red family
    pub pink: ColorScale,
    pub red: ColorScale,
    pub maroon: ColorScale,

    // Orange family
    pub peach: ColorScale,
    pub orange: ColorScale,
    pub rust: ColorScale,

    // Yellow family
    pub cream: ColorScale,
    pub yellow: ColorScale,
    pub gold: ColorScale,

    // Green family
    pub mint: ColorScale,
    pub green: ColorScale,
    pub forest: ColorScale,

    // Blue family
    pub sky: ColorScale,
    pub blue: ColorScale,
    pub navy: ColorScale,

    // Purple family
    pub lavender: ColorScale,
    pub purple: ColorScale,
    pub indigo: ColorScale,

    // Brown family
    pub tan: ColorScale,
    pub brown: ColorScale,
    pub chocolate: ColorScale,

    // Gray family
    pub silver: ColorScale,
    pub gray: ColorScale,
    pub charcoal: ColorScale,

    // Standalone
    pub white: ColorScale,
    pub black: ColorScale,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Red family
            pink: scale(Color::hex(0xF06292)),
            red: scale(Color::hex(0xE53935)),
            maroon: scale(Color::hex(0x7B1A1A)),

            // Orange family
            peach: scale(Color::hex(0xFFAB91)),
            orange: scale(Color::hex(0xFB8C00)),
            rust: scale(Color::hex(0xBF360C)),

            // Yellow family
            cream: scale(Color::hex(0xFFF9C4)),
            yellow: scale(Color::hex(0xFDD835)),
            gold: scale(Color::hex(0xF9A825)),

            // Green family
            mint: scale(Color::hex(0xA5D6A7)),
            green: scale(Color::hex(0x43A047)),
            forest: scale(Color::hex(0x1B5E20)),

            // Blue family
            sky: scale(Color::hex(0x81D4FA)),
            blue: scale(Color::hex(0x1E88E5)),
            navy: scale(Color::hex(0x0D47A1)),

            // Purple family
            lavender: scale(Color::hex(0xCE93D8)),
            purple: scale(Color::hex(0x8E24AA)),
            indigo: scale(Color::hex(0x4A148C)),

            // Brown family
            tan: scale(Color::hex(0xD7CCC8)),
            brown: scale(Color::hex(0x795548)),
            chocolate: scale(Color::hex(0x3E2723)),

            // Gray family
            silver: scale(Color::hex(0xBDBDBD)),
            gray: scale(Color::hex(0x757575)),
            charcoal: scale(Color::hex(0x424242)),

            // Standalone
            white: scale(Color::hex(0xFFFFFF)),
            black: scale(Color::hex(0x212121)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_red() {
        let c = Color::hex(0xFF0000);
        assert!((c.r - 1.0).abs() < 1e-4);
        assert!(c.g < 0.01);
        assert!(c.b < 0.01);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn rgb8_matches_hex() {
        let a = Color::hex(0x4080C0);
        let b = Color::rgb8(0x40, 0x80, 0xC0);
        assert!((a.r - b.r).abs() < 1e-5);
        assert!((a.g - b.g).abs() < 1e-5);
        assert!((a.b - b.b).abs() < 1e-5);
    }

    #[test]
    fn alpha_preserves_rgb() {
        let c = Color::hex(0xFF8000).alpha(0.5);
        let base = Color::hex(0xFF8000);
        assert_eq!(c.r, base.r);
        assert_eq!(c.g, base.g);
        assert_eq!(c.b, base.b);
        assert_eq!(c.a, 0.5);
    }

    #[test]
    fn linear_passthrough() {
        let c = Color::linear(0.3, 0.5, 0.7, 0.9);
        assert_eq!(c.r, 0.3);
        assert_eq!(c.g, 0.5);
        assert_eq!(c.b, 0.7);
        assert_eq!(c.a, 0.9);
    }

    #[test]
    fn srgb_to_linear_boundaries() {
        assert_eq!(srgb_to_linear(0.0), 0.0);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-5);
        assert!(srgb_to_linear(0.5) < 0.5);
    }

    #[test]
    fn scale_ordering() {
        let s = scale(Color::hex(0xFF0000));
        assert!(s.darker.r < s.dark.r);
        assert!(s.dark.r < s.base.r);
        assert!(s.base.r <= s.light.r);
        assert!(s.light.r <= s.lighter.r);
    }

    #[test]
    fn theme_default_creates() {
        let t = Theme::default();
        assert!(t.red.base.r > t.red.base.g);
        assert!(t.blue.base.b > t.blue.base.r);
    }
}
