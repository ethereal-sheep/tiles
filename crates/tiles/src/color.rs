#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
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

    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: srgb_to_linear(r as f32 / 255.0),
            g: srgb_to_linear(g as f32 / 255.0),
            b: srgb_to_linear(b as f32 / 255.0),
            a: a as f32 / 255.0,
        }
    }

    pub fn alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[u8; 3]> for Color {
    fn from(arr: [u8; 3]) -> Self {
        Self::rgb8(arr[0], arr[1], arr[2])
    }
}

impl From<[u8; 4]> for Color {
    fn from(arr: [u8; 4]) -> Self {
        Self::rgba8(arr[0], arr[1], arr[2], arr[3])
    }
}

pub(crate) fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
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
    fn rgba8_matches_rgb8_with_alpha() {
        let a = Color::rgba8(0x40, 0x80, 0xC0, 128);
        let b = Color::rgb8(0x40, 0x80, 0xC0);
        assert_eq!(a.r, b.r);
        assert_eq!(a.g, b.g);
        assert_eq!(a.b, b.b);
        assert!((a.a - 128.0 / 255.0).abs() < 1e-5);
    }

    #[test]
    fn from_u8_3_array_is_opaque_rgb8() {
        let c: Color = [0x40, 0x80, 0xC0].into();
        let expected = Color::rgb8(0x40, 0x80, 0xC0);
        assert_eq!(c, expected);
    }

    #[test]
    fn from_u8_4_array_matches_rgba8() {
        let c: Color = [0x40, 0x80, 0xC0, 128].into();
        let expected = Color::rgba8(0x40, 0x80, 0xC0, 128);
        assert_eq!(c, expected);
    }
}
