use tiles::Cell;

use crate::config::PaneStyle;
use crate::layout::Rect;

pub trait Widget {
    fn size(&self, style: &PaneStyle) -> (f32, f32);
    fn render(&self, rect: Rect, style: &PaneStyle, hovered: bool, active: bool) -> Vec<Cell>;
}
