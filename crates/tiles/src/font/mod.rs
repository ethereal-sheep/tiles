mod generated;

pub use generated::*;

pub struct Glyph {
    pub width: u8,
    pub height: u8,
    pub top: usize,
    pub bytes_per_row: usize,
    pub data: &'static [u8],
}

impl Glyph {
    pub fn pixel(&self, col: usize, row: usize) -> bool {
        if col >= self.width as usize || row >= self.height as usize {
            return false;
        }
        let byte_index = row * self.bytes_per_row + col / 8;
        let bit_index = 7 - (col % 8);
        if byte_index >= self.data.len() {
            return false;
        }
        (self.data[byte_index] >> bit_index) & 1 == 1
    }
}

pub struct Font {
    pub height: usize,
    pub default_gap: usize,
    glyphs: &'static [Glyph],
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Font")
    }
}

impl Font {
    pub const fn new(
        height: usize,
        default_gap: usize,
        glyphs: &'static [Glyph],
    ) -> Self {
        Self {
            height,
            default_gap,
            glyphs,
        }
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        let code = ch as u32;
        if code < 32 || code > 126 {
            return None;
        }
        let index = (code - 32) as usize;
        Some(&self.glyphs[index])
    }

    pub fn glyph_width(&self, ch: char) -> usize {
        self.glyph(ch).map(|g| g.width as usize).unwrap_or(0)
    }

    pub fn char_advance(&self, ch: char) -> usize {
        let w = self.glyph_width(ch);
        if w == 0 {
            return 0;
        }
        w + self.default_gap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_width_in_range() {
        let w = TOM_THUMB_3X5.glyph_width('A');
        assert!(w > 0);
    }

    #[test]
    fn glyph_width_out_of_range() {
        assert_eq!(TOM_THUMB_3X5.glyph_width('\x01'), 0);
        assert_eq!(TOM_THUMB_3X5.glyph_width('\u{FFFF}'), 0);
    }

    #[test]
    fn glyph_returns_some_for_printable_ascii() {
        let font = &MONO_5X7;
        for ch in ' '..='~' {
            assert!(font.glyph(ch).is_some(), "Missing glyph for '{ch}'");
        }
    }

    #[test]
    fn glyph_returns_none_out_of_range() {
        assert!(TOM_THUMB_3X5.glyph('\x00').is_none());
        assert!(TOM_THUMB_3X5.glyph('\u{FF}').is_none());
    }

    #[test]
    fn glyph_has_tight_dimensions() {
        let font = &TOM_THUMB_3X5;
        let g = font.glyph('A').unwrap();
        assert!(g.width > 0);
        assert!(g.height > 0);
        assert!(g.height as usize <= font.height);
    }

    #[test]
    fn pixel_reads_correctly() {
        let font = &TOM_THUMB_3X5;
        let g = font.glyph('I').unwrap();
        let mut has_set_pixel = false;
        for row in 0..g.height as usize {
            for col in 0..g.width as usize {
                if g.pixel(col, row) {
                    has_set_pixel = true;
                }
            }
        }
        assert!(has_set_pixel, "'I' should have at least one pixel set");
    }

    #[test]
    fn pixel_bounds_check() {
        let font = &MONO_5X7;
        let g = font.glyph('A').unwrap();
        assert!(!g.pixel(100, 0));
        assert!(!g.pixel(0, 100));
    }

    #[test]
    fn space_glyph_has_no_lit_pixels() {
        let font = &MONO_5X7;
        let g = font.glyph(' ').unwrap();
        assert_eq!(g.height, 0, "Space has no lit rows");
        for row in 0..g.height as usize {
            for col in 0..g.width as usize {
                assert!(!g.pixel(col, row), "Space should have no lit pixels");
            }
        }
    }

    #[test]
    fn default_gap_exists() {
        let _ = TOM_THUMB_3X5.default_gap;
        let _ = MONO_5X7.default_gap;
    }

    #[test]
    fn char_advance_is_width_plus_gap() {
        let font = &TOM_THUMB_3X5;
        let g = font.glyph('A').unwrap();
        assert_eq!(font.char_advance('A'), g.width as usize + font.default_gap);
    }

    #[test]
    fn char_advance_mono_uniform() {
        let font = &MONO_5X7;
        let adv_a = font.char_advance('A');
        let adv_i = font.char_advance('I');
        assert_eq!(adv_a, adv_i, "Mono font should have uniform advance");
    }

    #[test]
    fn tight_data_has_correct_length() {
        let font = &MONO_5X7;
        let g = font.glyph('A').unwrap();
        let expected_len = g.bytes_per_row * g.height as usize;
        assert_eq!(g.data.len(), expected_len);
    }
}
