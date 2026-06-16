use tiles::{Cell, InputState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Row,
    Column,
}

#[derive(Clone, Debug)]
struct LayoutCtx {
    origin: (i32, i32),
    cursor: (i32, i32),
    available: (i32, i32),
    axis: Axis,
    gap: i32,
    max_cross: i32,
}

impl LayoutCtx {
    fn advance(&mut self, w: i32, h: i32) -> (i32, i32) {
        let pos = self.cursor;
        match self.axis {
            Axis::Row => {
                self.cursor.0 += w + self.gap;
                self.max_cross = self.max_cross.max(h);
            }
            Axis::Column => {
                self.cursor.1 += h + self.gap;
                self.max_cross = self.max_cross.max(w);
            }
        }
        pos
    }

    fn used(&self) -> (i32, i32) {
        let main = match self.axis {
            Axis::Row => self.cursor.0 - self.origin.0,
            Axis::Column => self.cursor.1 - self.origin.1,
        };
        let main_trimmed = if main > 0 { main - self.gap } else { 0 };
        match self.axis {
            Axis::Row => (main_trimmed, self.max_cross),
            Axis::Column => (self.max_cross, main_trimmed),
        }
    }
}

pub struct UiBuffer {
    cells: Vec<Cell>,
    stack: Vec<LayoutCtx>,
    pub input: InputState,
}

impl UiBuffer {
    pub fn new() -> Self {
        Self {
            cells: Vec::with_capacity(256),
            stack: Vec::with_capacity(16),
            input: InputState::new(),
        }
    }

    pub fn begin(&mut self, screen_w: i32, screen_h: i32) {
        self.cells.clear();
        self.stack.clear();
        self.stack.push(LayoutCtx {
            origin: (0, 0),
            cursor: (0, 0),
            available: (screen_w, screen_h),
            axis: Axis::Column,
            gap: 0,
            max_cross: 0,
        });
    }

    pub fn alloc(&mut self, w: i32, h: i32) -> (i32, i32) {
        self.stack.last_mut().unwrap().advance(w, h)
    }

    pub fn available(&self) -> (i32, i32) {
        self.stack.last().unwrap().available
    }

    pub fn cursor(&self) -> (i32, i32) {
        self.stack.last().unwrap().cursor
    }

    pub fn push_layout(&mut self, origin: (i32, i32), available: (i32, i32), axis: Axis, gap: i32) {
        self.stack.push(LayoutCtx {
            origin,
            cursor: origin,
            available,
            axis,
            gap,
            max_cross: 0,
        });
    }

    pub fn pop_layout(&mut self) -> (i32, i32) {
        let ctx = self.stack.pop().unwrap();
        ctx.used()
    }

    pub fn advance(&mut self, w: i32, h: i32) {
        self.stack.last_mut().unwrap().advance(w, h);
    }

    pub fn push_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn reserve(&mut self) -> usize {
        let idx = self.cells.len();
        self.cells.push(Cell::new(0.0, 0.0));
        idx
    }

    pub fn patch(&mut self, idx: usize, cell: Cell) {
        self.cells[idx] = cell;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_resets_state() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        assert_eq!(buf.cursor(), (0, 0));
        assert_eq!(buf.available(), (256, 256));
        assert_eq!(buf.cells().len(), 0);
    }

    #[test]
    fn alloc_advances_cursor_column() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let pos = buf.alloc(10, 5);
        assert_eq!(pos, (0, 0));
        assert_eq!(buf.cursor(), (0, 5));
    }

    #[test]
    fn alloc_advances_cursor_row() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.push_layout((0, 0), (256, 256), Axis::Row, 0);
        let pos = buf.alloc(10, 5);
        assert_eq!(pos, (0, 0));
        assert_eq!(buf.cursor(), (10, 0));
    }

    #[test]
    fn gap_between_allocs() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.push_layout((0, 0), (256, 256), Axis::Row, 4);
        buf.alloc(10, 5);
        let pos2 = buf.alloc(10, 5);
        assert_eq!(pos2, (14, 0));
    }

    #[test]
    fn nested_layout() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        buf.push_layout((10, 10), (100, 100), Axis::Column, 2);
        buf.alloc(20, 8);
        buf.alloc(20, 8);
        let used = buf.pop_layout();
        assert_eq!(used, (20, 18));
        buf.advance(used.0, used.1);
        assert_eq!(buf.cursor(), (0, 18));
    }

    #[test]
    fn reserve_and_patch() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let slot = buf.reserve();
        buf.push_cell(Cell::new(5.0, 5.0));
        buf.patch(slot, Cell::new(0.0, 0.0).color(tiles::Color::linear(1.0, 0.0, 0.0, 1.0)));
        assert_eq!(buf.cells().len(), 2);
        assert_eq!(buf.cells()[0].color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn multiple_allocs_column() {
        let mut buf = UiBuffer::new();
        buf.begin(256, 256);
        let p1 = buf.alloc(10, 10);
        let p2 = buf.alloc(10, 10);
        let p3 = buf.alloc(10, 10);
        assert_eq!(p1, (0, 0));
        assert_eq!(p2, (0, 10));
        assert_eq!(p3, (0, 20));
    }
}
