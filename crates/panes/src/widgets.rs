use tiles::{Cell, Color, HitState, Rect, test_shape};

use crate::UiBuffer;
pub use crate::Axis;

pub fn fill_rect(buf: &mut UiBuffer, w: i32, h: i32, color: Color) {
    let (x, y) = buf.alloc(w, h);
    for dy in 0..h {
        for dx in 0..w {
            buf.push_cell(Cell::new((x + dx) as f32, (y + dy) as f32).color(color));
        }
    }
}

pub fn spacer(buf: &mut UiBuffer, w: i32, h: i32) {
    buf.alloc(w, h);
}

pub fn panel(
    buf: &mut UiBuffer,
    padding: i32,
    color: Color,
    axis: Axis,
    gap: i32,
    children: impl FnOnce(&mut UiBuffer),
) {
    let origin = buf.cursor();
    let avail = buf.available();

    buf.push_layout(
        (origin.0 + padding, origin.1 + padding),
        (avail.0 - padding * 2, avail.1 - padding * 2),
        axis,
        gap,
    );

    children(buf);

    let (inner_w, inner_h) = buf.pop_layout();
    let w = inner_w + padding * 2;
    let h = inner_h + padding * 2;

    buf.advance(w, h);

    let ox = origin.0;
    let oy = origin.1;
    for dy in 0..h {
        for dx in 0..w {
            buf.push_cell(
                Cell::new_3d((ox + dx) as f32, (oy + dy) as f32, -1.0).color(color),
            );
        }
    }
}

pub fn button(buf: &mut UiBuffer, w: i32, h: i32, color: Color, hover_color: Color) -> HitState {
    let (x, y) = buf.alloc(w, h);
    let rect = Rect::from_top_left(x as f32, y as f32, w as u32, h as u32);
    let hit = test_shape(&buf.input, &rect, true);

    let c = if hit.is_hovered() { hover_color } else { color };
    for dy in 0..h {
        for dx in 0..w {
            buf.push_cell(Cell::new((x + dx) as f32, (y + dy) as f32).color(c));
        }
    }

    hit
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Color = Color::linear(1.0, 0.0, 0.0, 1.0);
    const BLUE: Color = Color::linear(0.0, 0.0, 1.0, 1.0);
    const GREY: Color = Color::linear(0.3, 0.3, 0.3, 1.0);

    #[test]
    fn fill_rect_emits_cells() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        fill_rect(&mut buf, 3, 2, RED);
        assert_eq!(buf.cells().len(), 6);
        assert_eq!(buf.cells()[0].color, RED.to_array());
    }

    #[test]
    fn fill_rect_advances_cursor() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        fill_rect(&mut buf, 10, 5, RED);
        assert_eq!(buf.cursor(), (0, 5));
    }

    #[test]
    fn spacer_no_cells() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        spacer(&mut buf, 10, 10);
        assert_eq!(buf.cells().len(), 0);
        assert_eq!(buf.cursor(), (0, 10));
    }

    #[test]
    fn panel_wraps_children() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        panel(&mut buf, 2, GREY, Axis::Column, 0, |buf| {
            fill_rect(buf, 4, 3, RED);
        });
        // Panel size: (4+4)x(3+4) = 8x7
        // Child cells: 4*3=12, bg cells: 8*7=56, total=68
        assert_eq!(buf.cells().len(), 12 + 56);
        assert_eq!(buf.cursor(), (0, 7));
    }

    #[test]
    fn panel_positions_child_with_padding() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        panel(&mut buf, 3, GREY, Axis::Column, 0, |buf| {
            fill_rect(buf, 2, 2, BLUE);
        });
        // Child at (3,3), first child cell is index 0
        let child_cell = &buf.cells()[0];
        assert_eq!(child_cell.position.x, 3.0);
        assert_eq!(child_cell.position.y, 3.0);
    }

    #[test]
    fn panel_bg_behind_children() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        panel(&mut buf, 1, GREY, Axis::Column, 0, |buf| {
            fill_rect(buf, 2, 2, RED);
        });
        // Child cells first (z=0), bg cells after (z=-1)
        let child = &buf.cells()[0];
        let bg = buf.cells().last().unwrap();
        assert_eq!(child.position.z, 0.0);
        assert_eq!(bg.position.z, -1.0);
    }

    #[test]
    fn button_emits_cells_and_advances() {
        use glam::Vec2;
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.input.mouse_screen_pos = Vec2::new(100.0, 100.0);
        buf.input.prev_mouse_screen_pos = Vec2::new(100.0, 100.0);
        let hit = button(&mut buf, 5, 3, RED, BLUE);
        assert_eq!(buf.cells().len(), 15);
        assert_eq!(buf.cursor(), (0, 3));
        assert!(!hit.is_hovered());
    }

    #[test]
    fn button_hover_changes_color() {
        use glam::Vec2;
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.input.mouse_screen_pos = Vec2::new(2.0, 1.0);
        buf.input.prev_mouse_screen_pos = Vec2::new(2.0, 1.0);
        let hit = button(&mut buf, 5, 3, RED, BLUE);
        assert!(hit.is_hovered());
        assert_eq!(buf.cells()[0].color, BLUE.to_array());
    }

    #[test]
    fn button_click() {
        use glam::Vec2;
        use tiles::{ButtonState, MouseButton};
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.input.mouse_screen_pos = Vec2::new(2.0, 1.0);
        buf.input.prev_mouse_screen_pos = Vec2::new(2.0, 1.0);
        buf.input.left_press_screen_pos = Vec2::new(2.0, 1.0);
        buf.input.left_press_world_pos = Vec2::new(2.0, 1.0);
        let left = buf.input.mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.held_duration = 0.05;
        left.press_count = 1;

        let hit = button(&mut buf, 5, 3, RED, BLUE);
        assert!(hit.is_clicked());
    }

    #[test]
    fn nested_panels() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        panel(&mut buf, 2, GREY, Axis::Column, 0, |buf| {
            panel(buf, 1, BLUE, Axis::Column, 0, |buf| {
                fill_rect(buf, 3, 3, RED);
            });
        });
        // Inner: 3+2 x 3+2 = 5x5. Outer: 5+4 x 5+4 = 9x9
        assert_eq!(buf.cursor(), (0, 9));
    }
}
