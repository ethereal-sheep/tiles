use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;
use crate::widget::Widget;

pub struct TextWidget {
    pub content: String,
}

impl Widget for TextWidget {
    fn size(&self, style: &PaneStyle) -> (f32, f32) {
        let text_w: f32 = self.content.chars().map(|ch| style.font.char_advance(ch) as f32).sum();
        let h = style.font.height as f32;
        (text_w, h)
    }

    fn render(&self, rect: Rect, style: &PaneStyle, _hovered: bool, _active: bool) -> Vec<Cell> {
        let mut cells = Vec::new();
        let color = style.text_color;
        let mut cursor_x = rect.x;

        for ch in self.content.chars() {
            if let Some(glyph) = style.font.glyph(ch) {
                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            cells.push(
                                Cell::new(cursor_x + col as f32, rect.y + glyph.top as f32 + row as f32)
                                    .rgba(color[0], color[1], color[2], color[3])
                            );
                        }
                    }
                }
            }
            cursor_x += style.font.char_advance(ch) as f32;
        }

        cells
    }
}
