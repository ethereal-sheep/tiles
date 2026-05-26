use crate::document::Canvas;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Fill,
    Eyedropper,
    Line,
    Rect,
}

pub fn flood_fill(canvas: &mut Canvas, start_x: usize, start_y: usize, new_index: u8) {
    let target = canvas.get(start_x, start_y);
    if target == new_index {
        return;
    }

    let mut stack = vec![(start_x, start_y)];
    while let Some((x, y)) = stack.pop() {
        if x >= canvas.width || y >= canvas.height {
            continue;
        }
        if canvas.get(x, y) != target {
            continue;
        }
        canvas.set(x, y, new_index);

        if x > 0 { stack.push((x - 1, y)); }
        if x + 1 < canvas.width { stack.push((x + 1, y)); }
        if y > 0 { stack.push((x, y - 1)); }
        if y + 1 < canvas.height { stack.push((x, y + 1)); }
    }
}

pub fn draw_line(canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, index: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && y >= 0 {
            canvas.set(x as usize, y as usize, index);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn draw_rect(canvas: &mut Canvas, x0: usize, y0: usize, x1: usize, y1: usize, index: u8, filled: bool) {
    let min_x = x0.min(x1);
    let max_x = x0.max(x1);
    let min_y = y0.min(y1);
    let max_y = y0.max(y1);

    if filled {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                canvas.set(x, y, index);
            }
        }
    } else {
        for x in min_x..=max_x {
            canvas.set(x, min_y, index);
            canvas.set(x, max_y, index);
        }
        for y in min_y..=max_y {
            canvas.set(min_x, y, index);
            canvas.set(max_x, y, index);
        }
    }
}
