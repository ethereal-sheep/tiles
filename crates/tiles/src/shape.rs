use std::collections::HashSet;

use crate::cell::Cell;
use crate::drawable::Drawable;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokePosition {
    Inner,
    Outer,
    Middle,
}

pub trait Shape: Sized {
    fn fill_cells(&self, f: &mut impl FnMut(f32, f32));
    fn stroke_cells(&self, f: &mut impl FnMut(f32, f32));
    fn offset(&self, amount: i32) -> Self;

    fn fill(self) -> Fill<Self> {
        Fill { inner: self }
    }

    fn stroke(self, width: u32, position: StrokePosition) -> Stroke<Self> {
        Stroke { inner: self, width, position }
    }
}

pub struct Fill<S> {
    inner: S,
}

impl<S: Shape> Drawable for Fill<S> {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        self.inner.fill_cells(&mut |x, y| f(Cell::new(x, y)));
    }
}

pub struct Stroke<S> {
    inner: S,
    width: u32,
    position: StrokePosition,
}

impl<S: Shape> Drawable for Stroke<S> {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        if self.width == 0 {
            return;
        }

        let mut seen = HashSet::new();
        match self.position {
            StrokePosition::Inner => {
                for i in 0..self.width as i32 {
                    self.inner.offset(-i).stroke_cells(&mut |x, y| {
                        let key = (x as i32, y as i32);
                        if seen.insert(key) {
                            f(Cell::new(x, y));
                        }
                    });
                }
            }
            StrokePosition::Outer => {
                for i in 1..=self.width as i32 {
                    self.inner.offset(i).stroke_cells(&mut |x, y| {
                        let key = (x as i32, y as i32);
                        if seen.insert(key) {
                            f(Cell::new(x, y));
                        }
                    });
                }
            }
            StrokePosition::Middle => {
                self.inner.stroke_cells(&mut |x, y| {
                    let key = (x as i32, y as i32);
                    if seen.insert(key) {
                        f(Cell::new(x, y));
                    }
                });
                let w = self.width as i32;
                let inner_layers = (w - 1) / 2;
                let outer_layers = w - 1 - inner_layers;
                for i in 1..=inner_layers {
                    self.inner.offset(-i).stroke_cells(&mut |x, y| {
                        let key = (x as i32, y as i32);
                        if seen.insert(key) {
                            f(Cell::new(x, y));
                        }
                    });
                }
                for i in 1..=outer_layers {
                    self.inner.offset(i).stroke_cells(&mut |x, y| {
                        let key = (x as i32, y as i32);
                        if seen.insert(key) {
                            f(Cell::new(x, y));
                        }
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rect::Rect;

    fn collect_positions(drawable: impl Drawable) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();
        drawable.emit_cells(&mut |cell| {
            cells.push((cell.position.x as i32, cell.position.y as i32));
        });
        cells.sort();
        cells
    }

    #[test]
    fn fill_rect_emits_all_interior_cells() {
        let r = Rect::from_top_left(2.0, 3.0, 3, 2);
        let cells = collect_positions(r.fill());
        assert_eq!(cells, vec![
            (2, 3), (2, 4),
            (3, 3), (3, 4),
            (4, 3), (4, 4),
        ]);
    }

    #[test]
    fn fill_zero_size_rect_emits_nothing() {
        let r = Rect::from_top_left(5.0, 5.0, 0, 0);
        let cells = collect_positions(r.fill());
        assert!(cells.is_empty());
    }

    #[test]
    fn stroke_rect_width_1_inner() {
        let r = Rect::from_top_left(0.0, 0.0, 4, 4);
        let cells = collect_positions(r.stroke(1, StrokePosition::Inner));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(3, 0)));
        assert!(cells.contains(&(0, 3)));
        assert!(cells.contains(&(3, 3)));
        assert!(!cells.contains(&(1, 1)));
        assert!(!cells.contains(&(2, 2)));
    }

    #[test]
    fn stroke_rect_width_1_outer() {
        let r = Rect::from_top_left(2.0, 2.0, 3, 3);
        let cells = collect_positions(r.stroke(1, StrokePosition::Outer));
        assert!(cells.contains(&(1, 2)));
        assert!(cells.contains(&(5, 2)));
        assert!(cells.contains(&(2, 1)));
        assert!(cells.contains(&(2, 5)));
        assert!(!cells.contains(&(3, 3)));
    }

    #[test]
    fn stroke_rect_width_2_inner() {
        let r = Rect::from_top_left(0.0, 0.0, 6, 6);
        let cells = collect_positions(r.stroke(2, StrokePosition::Inner));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(1, 1)));
        assert!(!cells.contains(&(3, 3)));
    }

    #[test]
    fn stroke_middle_straddles_boundary() {
        let r = Rect::from_top_left(5.0, 5.0, 5, 5);
        let cells = collect_positions(r.stroke(3, StrokePosition::Middle));
        assert!(cells.contains(&(5, 5)));
        assert!(cells.contains(&(6, 6)));
        assert!(cells.contains(&(4, 5)));
    }

    #[test]
    fn stroke_zero_width_emits_nothing() {
        let r = Rect::from_top_left(0.0, 0.0, 5, 5);
        let cells = collect_positions(r.stroke(0, StrokePosition::Inner));
        assert!(cells.is_empty());
    }
}
