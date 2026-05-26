mod generated;

pub use generated::*;

pub struct Font {
    pub height: usize,
    bytes_per_row: usize,
    widths: &'static [u8],
    glyphs: &'static [u8],
    first_char: u8,
    last_char: u8,
}

impl Font {
    pub const fn new(
        height: usize,
        bytes_per_row: usize,
        widths: &'static [u8],
        glyphs: &'static [u8],
        first_char: u8,
        last_char: u8,
    ) -> Self {
        Self {
            height,
            bytes_per_row,
            widths,
            glyphs,
            first_char,
            last_char,
        }
    }

    pub fn glyph_width(&self, ch: char) -> usize {
        let code = ch as u32;
        if code < self.first_char as u32 || code > self.last_char as u32 {
            return 0;
        }
        let index = (code - self.first_char as u32) as usize;
        self.widths[index] as usize
    }

    pub fn char_advance(&self, ch: char) -> usize {
        self.glyph_width(ch)
    }

    pub fn glyph(&self, ch: char) -> Option<&[u8]> {
        let code = ch as u32;
        if code < self.first_char as u32 || code > self.last_char as u32 {
            return None;
        }
        let index = (code - self.first_char as u32) as usize;
        let bytes_per_glyph = self.bytes_per_row * self.height;
        let start = index * bytes_per_glyph;
        let end = start + bytes_per_glyph;
        if end > self.glyphs.len() {
            return None;
        }
        Some(&self.glyphs[start..end])
    }

    pub fn pixel(&self, glyph_data: &[u8], col: usize, row: usize) -> bool {
        let raster_width = self.bytes_per_row * 8;
        if col >= raster_width || row >= self.height {
            return false;
        }
        let byte_index = row * self.bytes_per_row + col / 8;
        let bit_index = 7 - (col % 8);
        (glyph_data[byte_index] >> bit_index) & 1 == 1
    }
}
