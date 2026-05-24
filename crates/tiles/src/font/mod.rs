mod generated;

pub use generated::*;

pub struct Font {
    pub width: usize,
    pub height: usize,
    pub spacing: usize,
    glyphs: &'static [u8],
    first_char: u8,
    last_char: u8,
}

impl Font {
    pub const fn new(
        width: usize,
        height: usize,
        spacing: usize,
        glyphs: &'static [u8],
        first_char: u8,
        last_char: u8,
    ) -> Self {
        Self {
            width,
            height,
            spacing,
            glyphs,
            first_char,
            last_char,
        }
    }

    pub fn glyph(&self, ch: char) -> Option<&[u8]> {
        let code = ch as u32;
        if code < self.first_char as u32 || code > self.last_char as u32 {
            return None;
        }
        let index = (code - self.first_char as u32) as usize;
        let bytes_per_row = (self.width + 7) / 8;
        let bytes_per_glyph = bytes_per_row * self.height;
        let start = index * bytes_per_glyph;
        let end = start + bytes_per_glyph;
        if end > self.glyphs.len() {
            return None;
        }
        Some(&self.glyphs[start..end])
    }

    pub fn pixel(&self, glyph_data: &[u8], col: usize, row: usize) -> bool {
        if col >= self.width || row >= self.height {
            return false;
        }
        let bytes_per_row = (self.width + 7) / 8;
        let byte_index = row * bytes_per_row + col / 8;
        let bit_index = 7 - (col % 8);
        (glyph_data[byte_index] >> bit_index) & 1 == 1
    }

    pub fn char_advance(&self) -> usize {
        self.width + self.spacing
    }
}
