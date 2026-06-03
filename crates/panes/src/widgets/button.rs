use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;
use crate::widget::Widget;

pub struct ButtonWidget {
    pub label: String,
}

impl Widget for ButtonWidget {
    fn size(&self, style: &PaneStyle) -> (f32, f32) {
        let text_w: f32 = self.label.chars().map(|ch| style.font.char_advance(ch) as f32).sum();
        let w = text_w + style.padding * 2.0;
        let h = style.font.height as f32 + style.padding * 2.0;
        (w, h)
    }

    fn render(&self, rect: Rect, style: &PaneStyle, hovered: bool, active: bool) -> Vec<Cell> {
        let mut cells = Vec::new();

        let bg = if active {
            [0.4, 0.4, 0.5, 1.0]
        } else if hovered {
            [0.3, 0.3, 0.38, 1.0]
        } else {
            [0.22, 0.22, 0.28, 1.0]
        };

        // Background
        for y in 0..rect.h as u32 {
            for x in 0..rect.w as u32 {
                cells.push(
                    Cell::new(rect.x + x as f32, rect.y + y as f32)
                        .rgba(bg[0], bg[1], bg[2], bg[3])
                );
            }
        }

        // Text
        let text_x = rect.x + style.padding;
        let text_y = rect.y + style.padding;
        let color = style.text_color;
        let mut cursor_x = text_x;
        for ch in self.label.chars() {
            if let Some(glyph) = style.font.glyph(ch) {
                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            cells.push(
                                Cell::new(cursor_x + col as f32, text_y + glyph.top as f32 + row as f32)
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
