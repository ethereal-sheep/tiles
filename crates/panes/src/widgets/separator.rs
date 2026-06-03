use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;
use crate::widget::Widget;

pub struct SeparatorWidget;

impl SeparatorWidget {
    pub fn render_at_width(rect: Rect, width: f32, style: &PaneStyle) -> Vec<Cell> {
        let mut cells = Vec::new();
        let color = style.border_color;
        for x in 0..width as u32 {
            cells.push(
                Cell::new(rect.x + x as f32, rect.y)
                    .rgba(color[0], color[1], color[2], color[3])
            );
        }
        cells
    }
}

impl Widget for SeparatorWidget {
    fn size(&self, _style: &PaneStyle) -> (f32, f32) {
        (0.0, 1.0)
    }

    fn render(&self, _rect: Rect, _style: &PaneStyle, _hovered: bool, _active: bool) -> Vec<Cell> {
        Vec::new()
    }
}
