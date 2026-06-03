use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;
use crate::widget::Widget;

const TRACK_WIDTH: f32 = 50.0;

pub struct SliderWidget {
    pub label: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

impl SliderWidget {
    pub fn label_width(&self, style: &PaneStyle) -> f32 {
        self.label.chars().map(|ch| style.font.char_advance(ch) as f32).sum()
    }

    pub fn track_width(&self, _style: &PaneStyle) -> f32 {
        TRACK_WIDTH
    }
}

impl Widget for SliderWidget {
    fn size(&self, style: &PaneStyle) -> (f32, f32) {
        let label_w = self.label_width(style);
        let w = label_w + 2.0 + TRACK_WIDTH;
        let h = (style.font.height as f32).max(5.0);
        (w, h)
    }

    fn render(&self, rect: Rect, style: &PaneStyle, hovered: bool, _active: bool) -> Vec<Cell> {
        let mut cells = Vec::new();
        let color = style.text_color;

        // Label
        let mut cursor_x = rect.x;
        for ch in self.label.chars() {
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

        // Track (3 pixels tall, centered)
        let track_x = rect.x + self.label_width(style) + 2.0;
        let track_center_y = rect.y + (rect.h / 2.0).floor();
        let track_color = if hovered {
            [0.35, 0.35, 0.4, 1.0]
        } else {
            [0.25, 0.25, 0.3, 1.0]
        };

        for dy in -1..=1_i32 {
            for x in 0..TRACK_WIDTH as u32 {
                cells.push(
                    Cell::new(track_x + x as f32, track_center_y + dy as f32)
                        .rgba(track_color[0], track_color[1], track_color[2], track_color[3])
                );
            }
        }

        // Filled portion
        let range = self.max - self.min;
        let t = if range > 0.0 { (self.value - self.min) / range } else { 0.0 };
        let fill_w = (t * TRACK_WIDTH) as u32;
        let fill_color = [0.4, 0.55, 0.8, 1.0];
        for dy in -1..=1_i32 {
            for x in 0..fill_w {
                cells.push(
                    Cell::new(track_x + x as f32, track_center_y + dy as f32)
                        .rgba(fill_color[0], fill_color[1], fill_color[2], fill_color[3])
                );
            }
        }

        // Handle (5 pixels tall)
        let handle_x = track_x + (t * (TRACK_WIDTH - 1.0)).round();
        let handle_color = [0.9, 0.9, 0.95, 1.0];
        for dy in -2..=2_i32 {
            cells.push(
                Cell::new(handle_x, track_center_y + dy as f32)
                    .rgba(handle_color[0], handle_color[1], handle_color[2], handle_color[3])
            );
        }

        cells
    }
}
