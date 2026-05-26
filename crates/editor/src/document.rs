use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Document {
    pub canvas: Canvas,
    pub palette: Palette,
    pub grid: Option<Grid>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Palette {
    pub colors: Vec<[f32; 4]>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Grid {
    pub tile_size: usize,
}

impl Document {
    pub fn new_tilesheet(tile_size: usize, cols: usize, rows: usize, palette: Palette) -> Self {
        let width = tile_size * cols;
        let height = tile_size * rows;
        Self {
            canvas: Canvas::new(width, height),
            palette,
            grid: Some(Grid { tile_size }),
        }
    }

    pub fn new_image(width: usize, height: usize, palette: Palette) -> Self {
        Self {
            canvas: Canvas::new(width, height),
            palette,
            grid: None,
        }
    }

    pub fn tile_count(&self) -> usize {
        if let Some(grid) = &self.grid {
            let cols = self.canvas.width / grid.tile_size;
            let rows = self.canvas.height / grid.tile_size;
            cols * rows
        } else {
            1
        }
    }

    pub fn tile_cols(&self) -> usize {
        if let Some(grid) = &self.grid {
            self.canvas.width / grid.tile_size
        } else {
            1
        }
    }

    pub fn tile_rows(&self) -> usize {
        if let Some(grid) = &self.grid {
            self.canvas.height / grid.tile_size
        } else {
            1
        }
    }
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![0; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            0
        }
    }

    pub fn set(&mut self, x: usize, y: usize, index: u8) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = index;
        }
    }
}

impl Palette {
    pub fn default_16() -> Self {
        Self {
            colors: vec![
                [0.0, 0.0, 0.0, 0.0],       // 0: transparent
                [0.0, 0.0, 0.0, 1.0],       // 1: black
                [1.0, 1.0, 1.0, 1.0],       // 2: white
                [0.9, 0.2, 0.2, 1.0],       // 3: red
                [0.2, 0.8, 0.2, 1.0],       // 4: green
                [0.2, 0.4, 0.9, 1.0],       // 5: blue
                [0.9, 0.9, 0.2, 1.0],       // 6: yellow
                [0.9, 0.5, 0.1, 1.0],       // 7: orange
                [0.6, 0.2, 0.8, 1.0],       // 8: purple
                [0.4, 0.8, 0.9, 1.0],       // 9: cyan
                [0.9, 0.5, 0.7, 1.0],       // 10: pink
                [0.4, 0.3, 0.2, 1.0],       // 11: brown
                [0.5, 0.5, 0.5, 1.0],       // 12: gray
                [0.7, 0.7, 0.7, 1.0],       // 13: light gray
                [0.2, 0.3, 0.15, 1.0],      // 14: dark green
                [0.15, 0.15, 0.3, 1.0],     // 15: dark blue
            ],
        }
    }

    pub fn color_at(&self, index: u8) -> [f32; 4] {
        if (index as usize) < self.colors.len() {
            self.colors[index as usize]
        } else {
            [1.0, 0.0, 1.0, 1.0] // magenta = missing
        }
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn add(&mut self, color: [f32; 4]) -> Option<u8> {
        if self.colors.len() >= 256 {
            return None;
        }
        let idx = self.colors.len() as u8;
        self.colors.push(color);
        Some(idx)
    }

    pub fn set_color(&mut self, index: u8, color: [f32; 4]) {
        if (index as usize) < self.colors.len() && index > 0 {
            self.colors[index as usize] = color;
        }
    }

    pub fn remove(&mut self, index: u8) {
        if index > 0 && (index as usize) < self.colors.len() {
            self.colors.remove(index as usize);
        }
    }
}

impl Grid {
    pub fn tile_origin(&self, tile_index: usize, sheet_cols: usize) -> (usize, usize) {
        let col = tile_index % sheet_cols;
        let row = tile_index / sheet_cols;
        (col * self.tile_size, row * self.tile_size)
    }
}
