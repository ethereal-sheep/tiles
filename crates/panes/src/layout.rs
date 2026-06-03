use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        pos.x >= self.x && pos.x < self.x + self.w && pos.y >= self.y && pos.y < self.y + self.h
    }

    pub fn top_left(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn clipped_to(&self, bounds: &Rect) -> Option<Rect> {
        let x0 = self.x.max(bounds.x);
        let y0 = self.y.max(bounds.y);
        let x1 = (self.x + self.w).min(bounds.x + bounds.w);
        let y1 = (self.y + self.h).min(bounds.y + bounds.h);
        if x1 > x0 && y1 > y0 {
            Some(Rect::new(x0, y0, x1 - x0, y1 - y0))
        } else {
            None
        }
    }
}

pub(crate) struct Cursor {
    pub origin: Vec2,
    pub pos: Vec2,
    pub row_height: f32,
    pub row_start_x: f32,
    pub same_line: bool,
    pub spacing: f32,
    pub max_row_width: f32,
}

impl Cursor {
    pub fn new(origin: Vec2, spacing: f32) -> Self {
        Self {
            origin,
            pos: origin,
            row_height: 0.0,
            row_start_x: origin.x,
            same_line: false,
            spacing,
            max_row_width: 0.0,
        }
    }

    pub fn allocate(&mut self, w: f32, h: f32) -> Rect {
        if self.same_line {
            self.same_line = false;
        } else {
            // Finish previous row — record its width
            let row_w = self.pos.x - self.row_start_x - self.spacing;
            if row_w > 0.0 {
                self.max_row_width = self.max_row_width.max(row_w);
            }
            if self.row_height > 0.0 {
                self.pos.y += self.row_height + self.spacing;
            }
            self.pos.x = self.row_start_x;
            self.row_height = 0.0;
        }

        let rect = Rect::new(self.pos.x, self.pos.y, w, h);
        self.pos.x += w + self.spacing;
        self.row_height = self.row_height.max(h);
        rect
    }

    pub fn set_same_line(&mut self) {
        self.same_line = true;
    }

    pub fn advance_vertical(&mut self, amount: f32) {
        if self.row_height > 0.0 {
            self.pos.y += self.row_height + self.spacing;
            self.row_height = 0.0;
        }
        self.pos.y += amount;
        self.pos.x = self.row_start_x;
    }

    pub fn advance_horizontal(&mut self, amount: f32) {
        self.pos.x += amount;
        self.same_line = true;
    }

    pub fn content_size(&self) -> Vec2 {
        // Include the current (last) row's width in the max calculation
        let current_row_w = self.pos.x - self.row_start_x - self.spacing;
        let width = self.max_row_width.max(current_row_w).max(0.0);
        let bottom = self.pos.y + self.row_height;
        Vec2::new(width, bottom - self.origin.y)
    }
}
