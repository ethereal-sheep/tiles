use crate::document::Document;

pub struct History {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
    max_depth: usize,
}

struct Snapshot {
    cells: Vec<u8>,
    palette_colors: Vec<[f32; 4]>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 100,
        }
    }

    pub fn save(&mut self, doc: &Document) {
        let snapshot = Snapshot {
            cells: doc.canvas.cells.clone(),
            palette_colors: doc.palette.colors.clone(),
        };
        self.undo_stack.push(snapshot);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            let current = Snapshot {
                cells: doc.canvas.cells.clone(),
                palette_colors: doc.palette.colors.clone(),
            };
            self.redo_stack.push(current);
            doc.canvas.cells = snapshot.cells;
            doc.palette.colors = snapshot.palette_colors;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, doc: &mut Document) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            let current = Snapshot {
                cells: doc.canvas.cells.clone(),
                palette_colors: doc.palette.colors.clone(),
            };
            self.undo_stack.push(current);
            doc.canvas.cells = snapshot.cells;
            doc.palette.colors = snapshot.palette_colors;
            true
        } else {
            false
        }
    }
}
