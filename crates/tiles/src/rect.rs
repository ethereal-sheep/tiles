use crate::shape::Shape;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    x: f32,
    y: f32,
    w: u32,
    h: u32,
}

impl Rect {
    pub fn from_top_left(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_top_right(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            x: x - w as f32,
            y,
            w,
            h,
        }
    }

    pub fn from_bottom_left(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            x,
            y: y - h as f32,
            w,
            h,
        }
    }

    pub fn from_bottom_right(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            x: x - w as f32,
            y: y - h as f32,
            w,
            h,
        }
    }

    pub fn from_corners(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        Self {
            x: min_x,
            y: min_y,
            w: (max_x - min_x) as u32,
            h: (max_y - min_y) as u32,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.w
    }

    pub fn height(&self) -> u32 {
        self.h
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w as f32
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h as f32
    }

    pub fn top_left(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn top_right(&self) -> (f32, f32) {
        (self.x + self.w as f32, self.y)
    }

    pub fn bottom_left(&self) -> (f32, f32) {
        (self.x, self.y + self.h as f32)
    }

    pub fn bottom_right(&self) -> (f32, f32) {
        (self.x + self.w as f32, self.y + self.h as f32)
    }

    pub fn expand(self, amount: i32) -> Self {
        let w = (self.w as i32 + amount * 2).max(0) as u32;
        let h = (self.h as i32 + amount * 2).max(0) as u32;
        Self {
            x: self.x - amount as f32,
            y: self.y - amount as f32,
            w,
            h,
        }
    }

    pub fn expand_top(self, amount: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - amount as f32,
            w: self.w,
            h: (self.h as i32 + amount).max(0) as u32,
        }
    }

    pub fn expand_bottom(self, amount: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: self.w,
            h: (self.h as i32 + amount).max(0) as u32,
        }
    }

    pub fn expand_left(self, amount: i32) -> Self {
        Self {
            x: self.x - amount as f32,
            y: self.y,
            w: (self.w as i32 + amount).max(0) as u32,
            h: self.h,
        }
    }

    pub fn expand_right(self, amount: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: (self.w as i32 + amount).max(0) as u32,
            h: self.h,
        }
    }

    pub fn rounded(self, radius: u32) -> RoundedRect {
        RoundedRect {
            x: self.x,
            y: self.y,
            w: self.w,
            h: self.h,
            radii: [radius; 4],
        }
    }

    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w as f32 && y >= self.y && y <= self.y + self.h as f32
    }
}

impl Shape for Rect {
    fn fill_cells(&self, f: &mut impl FnMut(f32, f32)) {
        let x0 = self.x.floor() as i32;
        let y0 = self.y.floor() as i32;

        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                f((x0 + x) as f32, (y0 + y) as f32);
            }
        }
    }

    fn stroke_cells(&self, f: &mut impl FnMut(f32, f32)) {
        if self.w == 0 || self.h == 0 {
            return;
        }

        let x0 = self.x.floor() as i32;
        let y0 = self.y.floor() as i32;
        let w = self.w as i32;
        let h = self.h as i32;

        for x in 0..w {
            f((x0 + x) as f32, y0 as f32);
            if h > 1 {
                f((x0 + x) as f32, (y0 + h - 1) as f32);
            }
        }
        for y in 1..(h - 1) {
            f(x0 as f32, (y0 + y) as f32);
            if w > 1 {
                f((x0 + w - 1) as f32, (y0 + y) as f32);
            }
        }
    }

    fn offset(&self, amount: i32) -> Self {
        self.expand(amount)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundedRect {
    x: f32,
    y: f32,
    w: u32,
    h: u32,
    radii: [u32; 4], // [top_left, top_right, bottom_right, bottom_left]
}

impl RoundedRect {
    pub fn top_left(mut self, radius: u32) -> Self {
        self.radii[0] = radius;
        self
    }

    pub fn top_right(mut self, radius: u32) -> Self {
        self.radii[1] = radius;
        self
    }

    pub fn bottom_right(mut self, radius: u32) -> Self {
        self.radii[2] = radius;
        self
    }

    pub fn bottom_left(mut self, radius: u32) -> Self {
        self.radii[3] = radius;
        self
    }

    pub fn radius(mut self, radius: u32) -> Self {
        self.radii = [radius; 4];
        self
    }

    fn clamped_radii(&self) -> [i32; 4] {
        let max_r = (self.w as i32 / 2).min(self.h as i32 / 2).max(0);
        [
            (self.radii[0] as i32).min(max_r),
            (self.radii[1] as i32).min(max_r),
            (self.radii[2] as i32).min(max_r),
            (self.radii[3] as i32).min(max_r),
        ]
    }

    fn contains(&self, px: i32, py: i32) -> bool {
        let w = self.w as i32;
        let h = self.h as i32;

        if px < 0 || px >= w || py < 0 || py >= h {
            return false;
        }

        let r = self.clamped_radii();

        // Top-left corner
        if px < r[0] && py < r[0] {
            let dx = px - r[0];
            let dy = py - r[0];
            return dx * dx + dy * dy <= r[0] * r[0];
        }
        // Top-right corner
        if px >= w - r[1] && py < r[1] {
            let dx = px - (w - r[1] - 1);
            let dy = py - r[1];
            return dx * dx + dy * dy <= r[1] * r[1];
        }
        // Bottom-right corner
        if px >= w - r[2] && py >= h - r[2] {
            let dx = px - (w - r[2] - 1);
            let dy = py - (h - r[2] - 1);
            return dx * dx + dy * dy <= r[2] * r[2];
        }
        // Bottom-left corner
        if px < r[3] && py >= h - r[3] {
            let dx = px - r[3];
            let dy = py - (h - r[3] - 1);
            return dx * dx + dy * dy <= r[3] * r[3];
        }

        true
    }
}

impl Shape for RoundedRect {
    fn fill_cells(&self, f: &mut impl FnMut(f32, f32)) {
        let x0 = self.x.floor() as i32;
        let y0 = self.y.floor() as i32;

        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                if self.contains(x, y) {
                    f((x0 + x) as f32, (y0 + y) as f32);
                }
            }
        }
    }

    fn stroke_cells(&self, f: &mut impl FnMut(f32, f32)) {
        let x0 = self.x.floor() as i32;
        let y0 = self.y.floor() as i32;
        let w = self.w as i32;
        let h = self.h as i32;

        for y in 0..h {
            for x in 0..w {
                if !self.contains(x, y) {
                    continue;
                }
                let is_border = x == 0
                    || x == w - 1
                    || y == 0
                    || y == h - 1
                    || !self.contains(x - 1, y)
                    || !self.contains(x + 1, y)
                    || !self.contains(x, y - 1)
                    || !self.contains(x, y + 1);
                if is_border {
                    f((x0 + x) as f32, (y0 + y) as f32);
                }
            }
        }
    }

    fn offset(&self, amount: i32) -> Self {
        let w = (self.w as i32 + amount * 2).max(0) as u32;
        let h = (self.h as i32 + amount * 2).max(0) as u32;
        Self {
            x: self.x - amount as f32,
            y: self.y - amount as f32,
            w,
            h,
            radii: [
                (self.radii[0] as i32 + amount).max(0) as u32,
                (self.radii[1] as i32 + amount).max(0) as u32,
                (self.radii[2] as i32 + amount).max(0) as u32,
                (self.radii[3] as i32 + amount).max(0) as u32,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_top_left_accessors() {
        let r = Rect::from_top_left(10.0, 20.0, 30, 40);
        assert_eq!(r.x(), 10.0);
        assert_eq!(r.y(), 20.0);
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn from_top_left_edges() {
        let r = Rect::from_top_left(10.0, 20.0, 30, 40);
        assert_eq!(r.left(), 10.0);
        assert_eq!(r.right(), 40.0);
        assert_eq!(r.top(), 20.0);
        assert_eq!(r.bottom(), 60.0);
    }

    #[test]
    fn from_top_left_corners() {
        let r = Rect::from_top_left(10.0, 20.0, 30, 40);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.top_right(), (40.0, 20.0));
        assert_eq!(r.bottom_left(), (10.0, 60.0));
        assert_eq!(r.bottom_right(), (40.0, 60.0));
    }

    #[test]
    fn from_top_right() {
        let r = Rect::from_top_right(40.0, 20.0, 30, 40);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.top_right(), (40.0, 20.0));
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn from_bottom_left() {
        let r = Rect::from_bottom_left(10.0, 60.0, 30, 40);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.bottom_left(), (10.0, 60.0));
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn from_bottom_right() {
        let r = Rect::from_bottom_right(40.0, 60.0, 30, 40);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.bottom_right(), (40.0, 60.0));
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn from_corners_ordered() {
        let r = Rect::from_corners(10.0, 20.0, 40.0, 60.0);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.bottom_right(), (40.0, 60.0));
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn from_corners_reversed() {
        let r = Rect::from_corners(40.0, 60.0, 10.0, 20.0);
        assert_eq!(r.top_left(), (10.0, 20.0));
        assert_eq!(r.bottom_right(), (40.0, 60.0));
        assert_eq!(r.width(), 30);
        assert_eq!(r.height(), 40);
    }

    #[test]
    fn zero_size_rect() {
        let r = Rect::from_top_left(5.0, 5.0, 0, 0);
        assert_eq!(r.width(), 0);
        assert_eq!(r.height(), 0);
        assert_eq!(r.top_left(), (5.0, 5.0));
        assert_eq!(r.bottom_right(), (5.0, 5.0));
    }

    #[test]
    fn expand_all_sides() {
        let r = Rect::from_top_left(10.0, 10.0, 10, 10).expand(2);
        assert_eq!(r.left(), 8.0);
        assert_eq!(r.top(), 8.0);
        assert_eq!(r.width(), 14);
        assert_eq!(r.height(), 14);
    }

    #[test]
    fn expand_clamps_to_zero() {
        let r = Rect::from_top_left(10.0, 10.0, 4, 4).expand(-5);
        assert_eq!(r.width(), 0);
        assert_eq!(r.height(), 0);
    }

    #[test]
    fn expand_individual_sides() {
        let r = Rect::from_top_left(10.0, 10.0, 10, 10).expand_right(5);
        assert_eq!(r.width(), 15);
        assert_eq!(r.left(), 10.0);

        let r = Rect::from_top_left(10.0, 10.0, 10, 10).expand_left(5);
        assert_eq!(r.width(), 15);
        assert_eq!(r.left(), 5.0);

        let r = Rect::from_top_left(10.0, 10.0, 10, 10).expand_top(5);
        assert_eq!(r.height(), 15);
        assert_eq!(r.top(), 5.0);

        let r = Rect::from_top_left(10.0, 10.0, 10, 10).expand_bottom(5);
        assert_eq!(r.height(), 15);
        assert_eq!(r.top(), 10.0);
    }

    #[test]
    fn expand_individual_clamps_to_zero() {
        let r = Rect::from_top_left(10.0, 10.0, 4, 4).expand_right(-10);
        assert_eq!(r.width(), 0);
    }

    #[test]
    fn rounded_creates_rounded_rect() {
        let r = Rect::from_top_left(0.0, 0.0, 20, 10).rounded(5);
        assert_eq!(r.radii, [5, 5, 5, 5]);
    }

    #[test]
    fn rounded_rect_per_corner() {
        let r = Rect::from_top_left(0.0, 0.0, 20, 10)
            .rounded(0)
            .top_left(3)
            .bottom_right(5);
        assert_eq!(r.radii, [3, 0, 5, 0]);
    }

    #[test]
    fn rounded_rect_offset_adjusts_radii() {
        let r = Rect::from_top_left(0.0, 0.0, 20, 10).rounded(4);
        let expanded = r.offset(2);
        assert_eq!(expanded.radii, [6, 6, 6, 6]);
        assert_eq!(expanded.w, 24);
        assert_eq!(expanded.h, 14);
    }

    #[test]
    fn rounded_rect_offset_clamps_radii_to_zero() {
        let r = Rect::from_top_left(0.0, 0.0, 20, 10).rounded(2);
        let shrunk = r.offset(-5);
        assert_eq!(shrunk.radii, [0, 0, 0, 0]);
    }

    #[test]
    fn rounded_rect_fill_excludes_corners() {
        let r = Rect::from_top_left(0.0, 0.0, 10, 10).rounded(4);
        let mut cells = Vec::new();
        r.fill_cells(&mut |x, y| cells.push((x as i32, y as i32)));
        assert!(!cells.contains(&(0, 0)));
        assert!(cells.contains(&(5, 5)));
    }

    #[test]
    fn rect_stroke_cells_is_perimeter() {
        let r = Rect::from_top_left(0.0, 0.0, 3, 3);
        let mut cells = Vec::new();
        r.stroke_cells(&mut |x, y| cells.push((x as i32, y as i32)));
        cells.sort();
        assert_eq!(cells.len(), 8);
        assert!(!cells.contains(&(1, 1)));
    }

    #[test]
    fn rect_fill_cells_count() {
        let r = Rect::from_top_left(0.0, 0.0, 5, 5);
        let mut count = 0;
        r.fill_cells(&mut |_, _| count += 1);
        assert_eq!(count, 25);
    }
}
