use crate::cell::Cell;
use crate::drawable::Drawable;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: u32,
}

impl Line {
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2, width: 1 }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }
}

impl Drawable for Line {
    fn emit_cells(&self, f: &mut impl FnMut(Cell)) {
        if self.width == 0 {
            return;
        }

        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let len = (dx * dx + dy * dy).sqrt();

        if len < 1e-6 {
            f(Cell::new(self.x1.floor(), self.y1.floor()));
            return;
        }

        // Perpendicular direction (normalized)
        let px = -dy / len;
        let py = dx / len;

        let steps = len.ceil() as i32 + 1;
        let width = self.width as i32;

        // Bias: for even widths, expand one extra in the positive perpendicular direction
        let neg_layers = (width - 1) / 2;
        let pos_layers = width - 1 - neg_layers;

        let mut seen = std::collections::HashSet::new();

        for i in 0..=steps {
            let t = (i as f32) / (steps as f32);
            let cx = self.x1 + dx * t;
            let cy = self.y1 + dy * t;

            for w in -(neg_layers)..=(pos_layers) {
                let x = (cx + px * w as f32).floor() as i32;
                let y = (cy + py * w as f32).floor() as i32;
                if seen.insert((x, y)) {
                    f(Cell::new(x as f32, y as f32));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect_positions(line: Line) -> Vec<(i32, i32)> {
        let mut cells = Vec::new();
        line.emit_cells(&mut |cell| {
            cells.push((cell.position.x as i32, cell.position.y as i32));
        });
        cells.sort();
        cells
    }

    #[test]
    fn horizontal_line() {
        let cells = collect_positions(Line::new(0.0, 0.0, 4.0, 0.0));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(4, 0)));
        for &(_, y) in &cells {
            assert_eq!(y, 0);
        }
    }

    #[test]
    fn vertical_line() {
        let cells = collect_positions(Line::new(0.0, 0.0, 0.0, 4.0));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(0, 4)));
        for &(x, _) in &cells {
            assert_eq!(x, 0);
        }
    }

    #[test]
    fn width_increases_cells() {
        let thin = collect_positions(Line::new(0.0, 0.0, 10.0, 0.0));
        let thick = collect_positions(Line::new(0.0, 0.0, 10.0, 0.0).width(3));
        assert!(thick.len() > thin.len());
    }

    #[test]
    fn zero_length_line_emits_one_cell() {
        let cells = collect_positions(Line::new(5.0, 5.0, 5.0, 5.0));
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0], (5, 5));
    }

    #[test]
    fn zero_width_emits_nothing() {
        let cells = collect_positions(Line::new(0.0, 0.0, 10.0, 0.0).width(0));
        assert!(cells.is_empty());
    }

    #[test]
    fn diagonal_line_covers_endpoints() {
        let cells = collect_positions(Line::new(0.0, 0.0, 5.0, 5.0));
        assert!(cells.contains(&(0, 0)));
        assert!(cells.contains(&(5, 5)));
    }

    #[test]
    fn even_width_biases_one_side() {
        let cells = collect_positions(Line::new(0.0, 5.0, 10.0, 5.0).width(2));
        let ys: std::collections::HashSet<_> = cells.iter().map(|&(_, y)| y).collect();
        assert_eq!(ys.len(), 2);
    }

    #[test]
    fn width_3_horizontal_is_symmetric() {
        let cells = collect_positions(Line::new(0.0, 5.0, 10.0, 5.0).width(3));
        let ys: std::collections::HashSet<_> = cells.iter().map(|&(_, y)| y).collect();
        assert!(ys.contains(&4));
        assert!(ys.contains(&5));
        assert!(ys.contains(&6));
    }
}
