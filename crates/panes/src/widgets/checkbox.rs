use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;
use crate::widget::Widget;

const BOX_SIZE: f32 = 7.0;

pub struct CheckboxWidget {
    pub label: String,
    pub checked: bool,
}

impl Widget for CheckboxWidget {
    fn size(&self, style: &PaneStyle) -> (f32, f32) {
        let label_w: f32 = self.label.chars().map(|ch| style.font.char_advance(ch) as f32).sum();
        let w = BOX_SIZE + 2.0 + label_w;
        let h = BOX_SIZE.max(style.font.height as f32);
        (w, h)
    }

    fn render(&self, rect: Rect, style: &PaneStyle, hovered: bool, _active: bool) -> Vec<Cell> {
        let mut cells = Vec::new();

        // Box outline
        let box_y = rect.y + ((rect.h - BOX_SIZE) / 2.0).floor();
        let border_color = if hovered {
            [0.6, 0.6, 0.7, 1.0]
        } else {
            [0.4, 0.4, 0.45, 1.0]
        };

        for x in 0..BOX_SIZE as u32 {
            cells.push(Cell::new(rect.x + x as f32, box_y).rgba(border_color[0], border_color[1], border_color[2], border_color[3]));
            cells.push(Cell::new(rect.x + x as f32, box_y + BOX_SIZE - 1.0).rgba(border_color[0], border_color[1], border_color[2], border_color[3]));
        }
        for y in 1..BOX_SIZE as u32 - 1 {
            cells.push(Cell::new(rect.x, box_y + y as f32).rgba(border_color[0], border_color[1], border_color[2], border_color[3]));
            cells.push(Cell::new(rect.x + BOX_SIZE - 1.0, box_y + y as f32).rgba(border_color[0], border_color[1], border_color[2], border_color[3]));
        }

        // Check mark (fill inner area)
        if self.checked {
            let check_color = [0.5, 0.7, 0.9, 1.0];
            for y in 1..BOX_SIZE as u32 - 1 {
                for x in 1..BOX_SIZE as u32 - 1 {
                    cells.push(
                        Cell::new(rect.x + x as f32, box_y + y as f32)
                            .rgba(check_color[0], check_color[1], check_color[2], check_color[3])
                    );
                }
            }
        }

        // Label
        let label_x = rect.x + BOX_SIZE + 2.0;
        let label_y = rect.y + ((rect.h - style.font.height as f32) / 2.0).floor();
        let color = style.text_color;
        let mut cursor_x = label_x;
        for ch in self.label.chars() {
            if let Some(glyph) = style.font.glyph(ch) {
                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            cells.push(
                                Cell::new(cursor_x + col as f32, label_y + glyph.top as f32 + row as f32)
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
