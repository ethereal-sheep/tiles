use glam::Vec2;
use tiles::{App, Cell, Color, KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent, State};
use tiles::font::{Font, TOM_THUMB_3X5};

use crate::document::{Document, Palette};
use crate::history::History;
use crate::tools::{self, Tool};

pub struct Panel {
    pub pos: Vec2,
    pub visible: bool,
}

impl Panel {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            visible: true,
        }
    }
}

pub struct Editor {
    pub doc: Document,
    history: History,
    tool: Tool,
    active_color: u8,
    active_tile: usize,
    // Panels
    canvas_panel: Panel,
    canvas_scale: u32,
    palette_panel: Panel,
    tools_panel: Panel,
    // Interaction state
    painting: bool,
    panning: bool,
    pan_anchor: Vec2,
    drag_start: Option<(usize, usize)>,
    last_paint_pos: Option<(usize, usize)>,
    space_held: bool,
}

impl Editor {
    pub fn new() -> Self {
        let doc = Document::new_tilesheet(16, 4, 4, Palette::default_16());
        let canvas_scale: u32 = 4;

        Self {
            doc,
            history: History::new(),
            tool: Tool::Pencil,
            active_color: 1,
            active_tile: 0,
            canvas_panel: Panel::new(0.0, 0.0),
            canvas_scale,
            palette_panel: Panel::new(0.0, 0.0),
            tools_panel: Panel::new(0.0, 0.0),
            painting: false,
            panning: false,
            pan_anchor: Vec2::ZERO,
            drag_start: None,
            last_paint_pos: None,
            space_held: false,
        }
    }

    fn layout(&mut self, vp: Vec2) {
        let scale = self.canvas_scale as f32;
        let canvas_w = self.doc.canvas.width as f32 * scale;
        let canvas_h = self.doc.canvas.height as f32 * scale;

        self.canvas_panel.pos = Vec2::new(-canvas_w / 2.0, canvas_h / 2.0);
        self.palette_panel.pos = Vec2::new(-vp.x / 2.0 + 2.0, vp.y / 2.0 - 2.0);
        self.tools_panel.pos = Vec2::new(vp.x / 2.0 - 60.0, vp.y / 2.0 - 2.0);
    }

    fn world_to_canvas(&self, world: Vec2) -> Option<(usize, usize)> {
        let scale = self.canvas_scale as f32;
        let local_x = world.x - self.canvas_panel.pos.x;
        let local_y = -(world.y - self.canvas_panel.pos.y);

        if local_x >= 0.0 && local_y >= 0.0 {
            let cx = (local_x / scale) as usize;
            let cy = (local_y / scale) as usize;
            if cx < self.doc.canvas.width && cy < self.doc.canvas.height {
                return Some((cx, cy));
            }
        }
        None
    }

    fn palette_hit(&self, world: Vec2) -> Option<u8> {
        let local_x = world.x - self.palette_panel.pos.x;
        let local_y = -(world.y - self.palette_panel.pos.y);

        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }

        let cols = 2;
        let swatch_size = 2.0;
        let col = (local_x / swatch_size) as usize;
        let row = (local_y / swatch_size) as usize;

        if col >= cols {
            return None;
        }

        let idx = row * cols + col;
        if idx < self.doc.palette.colors.len() {
            Some(idx as u8)
        } else {
            None
        }
    }

    fn apply_tool_at(&mut self, x: usize, y: usize) {
        match self.tool {
            Tool::Pencil => {
                self.doc.canvas.set(x, y, self.active_color);
            }
            Tool::Eraser => {
                self.doc.canvas.set(x, y, 0);
            }
            Tool::Fill => {
                tools::flood_fill(&mut self.doc.canvas, x, y, self.active_color);
            }
            Tool::Eyedropper => {
                self.active_color = self.doc.canvas.get(x, y);
            }
            Tool::Line | Tool::Rect => {
                self.drag_start = Some((x, y));
            }
        }
    }

    fn finish_drag(&mut self, end_x: usize, end_y: usize) {
        if let Some((sx, sy)) = self.drag_start.take() {
            match self.tool {
                Tool::Line => {
                    tools::draw_line(
                        &mut self.doc.canvas,
                        sx as i32, sy as i32,
                        end_x as i32, end_y as i32,
                        self.active_color,
                    );
                }
                Tool::Rect => {
                    tools::draw_rect(
                        &mut self.doc.canvas,
                        sx, sy, end_x, end_y,
                        self.active_color,
                        false,
                    );
                }
                _ => {}
            }
        }
    }

    fn draw_checkerboard(&self, state: &mut State) {
        let w = self.doc.canvas.width;
        let h = self.doc.canvas.height;
        let scale = self.canvas_scale as f32;
        let light = [0.25, 0.25, 0.25];
        let dark = [0.18, 0.18, 0.18];

        for y in 0..h {
            for x in 0..w {
                let color = if (x + y) % 2 == 0 { light } else { dark };
                let wx = self.canvas_panel.pos.x + x as f32 * scale;
                let wy = self.canvas_panel.pos.y - y as f32 * scale;
                for dy in 0..self.canvas_scale {
                    for dx in 0..self.canvas_scale {
                        state.draw(
                            Cell::new_3d(wx + dx as f32, wy - dy as f32, -1.0)
                                .color(Color::linear(color[0], color[1], color[2], 1.0)),
                        );
                    }
                }
            }
        }
    }

    fn draw_canvas(&self, state: &mut State) {
        let w = self.doc.canvas.width;
        let h = self.doc.canvas.height;
        let scale = self.canvas_scale as f32;

        for y in 0..h {
            for x in 0..w {
                let idx = self.doc.canvas.get(x, y);
                if idx == 0 {
                    continue;
                }
                let color = self.doc.palette.color_at(idx);
                let wx = self.canvas_panel.pos.x + x as f32 * scale;
                let wy = self.canvas_panel.pos.y - y as f32 * scale;
                for dy in 0..self.canvas_scale {
                    for dx in 0..self.canvas_scale {
                        state.draw(
                            Cell::new(wx + dx as f32, wy - dy as f32)
                                .color(Color::linear(color[0], color[1], color[2], color[3])),
                        );
                    }
                }
            }
        }
    }

    fn draw_palette(&self, state: &mut State) {
        let cols = 2;
        let swatch_size: u32 = 2;

        for (i, color) in self.doc.palette.colors.iter().enumerate() {
            let col = i % cols;
            let row = i / cols;
            let sx = self.palette_panel.pos.x + (col * swatch_size as usize) as f32;
            let sy = self.palette_panel.pos.y - (row * swatch_size as usize) as f32;

            for dy in 0..swatch_size {
                for dx in 0..swatch_size {
                    let mut cell = Cell::new(sx + dx as f32, sy - dy as f32)
                        .color(Color::linear(color[0], color[1], color[2], color[3]));
                    if i as u8 == self.active_color {
                        cell = Cell::new(sx + dx as f32, sy - dy as f32)
                            .color(Color::linear(color[0], color[1], color[2], color[3]))
                            .emissive();
                    }
                    state.draw(cell);
                }
            }
        }
    }

    fn draw_tools_panel(&self, state: &mut State) {
        let font = &TOM_THUMB_3X5;
        let tools_list = [
            ("Pencil", "B", Tool::Pencil),
            ("Eraser", "E", Tool::Eraser),
            ("Fill", "G", Tool::Fill),
            ("Eyedrop", "I", Tool::Eyedropper),
            ("Line", "L", Tool::Line),
            ("Rect", "R", Tool::Rect),
        ];

        let line_height = font.height as f32 + 2.0;

        for (i, (name, key, tool)) in tools_list.iter().enumerate() {
            let y = self.tools_panel.pos.y - i as f32 * line_height;
            let active = self.tool == *tool;

            let label = if active {
                format!("> {} [{}]", name, key)
            } else {
                format!("  {} [{}]", name, key)
            };

            let color = if active {
                [1.0, 1.0, 0.5, 1.0]
            } else {
                [0.7, 0.7, 0.7, 1.0]
            };

            self.draw_text(state, font, &label, self.tools_panel.pos.x, y, color);
        }

        let hotkeys_y = self.tools_panel.pos.y - tools_list.len() as f32 * line_height - line_height;
        let hints = [
            "Z     Undo",
            "Sh+Z  Redo",
            "1-0   Color",
            "Sp+Lm Pan",
            "Scrl  Zoom",
            "Ct+S  Save",
        ];

        for (i, hint) in hints.iter().enumerate() {
            let y = hotkeys_y - i as f32 * line_height;
            self.draw_text(state, font, hint, self.tools_panel.pos.x, y, [0.5, 0.5, 0.5, 1.0]);
        }
    }

    fn draw_text(&self, state: &mut State, font: &Font, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(glyph) = font.glyph(ch) {
                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            state.draw(
                                Cell::new(cursor_x + col as f32, y - (glyph.top as f32 + row as f32))
                                    .color(Color::linear(color[0], color[1], color[2], color[3])),
                            );
                        }
                    }
                }
            }
            cursor_x += font.char_advance(ch) as f32;
        }
    }
}

impl App for Editor {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.12, 0.12, 0.14, 1.0);
        state.set_window_background(0.05, 0.05, 0.07, 1.0);
        state.set_ambient_illumination(1.0);

        let vp = state.viewport_size();
        self.layout(vp);
    }

    fn update(&mut self, state: &mut State) {
        if self.panning {
            let pos = state.mouse_position();
            let delta = pos - self.pan_anchor;
            self.canvas_panel.pos += delta;
            self.pan_anchor = pos;
            return;
        }

        if self.painting && (self.tool == Tool::Pencil || self.tool == Tool::Eraser) {
            let pos = state.mouse_position();
            if let Some((x, y)) = self.world_to_canvas(pos) {
                if self.last_paint_pos != Some((x, y)) {
                    if self.tool == Tool::Pencil {
                        self.doc.canvas.set(x, y, self.active_color);
                    } else {
                        self.doc.canvas.set(x, y, 0);
                    }
                    self.last_paint_pos = Some((x, y));
                }
            }
        }
    }

    fn draw(&mut self, state: &mut State) {
        self.draw_checkerboard(state);
        self.draw_canvas(state);
        self.draw_palette(state);
        self.draw_tools_panel(state);
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        match event.state {
            KeyState::Pressed => match event.key {
                KeyCode::Space => self.space_held = true,
                KeyCode::Escape => state.quit = true,
                KeyCode::B => self.tool = Tool::Pencil,
                KeyCode::E => self.tool = Tool::Eraser,
                KeyCode::G => self.tool = Tool::Fill,
                KeyCode::I => self.tool = Tool::Eyedropper,
                KeyCode::L => self.tool = Tool::Line,
                KeyCode::R => self.tool = Tool::Rect,
                KeyCode::Z => {
                    if state.is_key_down(KeyCode::LShift) || state.is_key_down(KeyCode::RShift) {
                        self.history.redo(&mut self.doc);
                    } else {
                        self.history.undo(&mut self.doc);
                    }
                }
                KeyCode::Key1 => self.active_color = 1,
                KeyCode::Key2 => self.active_color = 2,
                KeyCode::Key3 => self.active_color = 3,
                KeyCode::Key4 => self.active_color = 4,
                KeyCode::Key5 => self.active_color = 5,
                KeyCode::Key6 => self.active_color = 6,
                KeyCode::Key7 => self.active_color = 7,
                KeyCode::Key8 => self.active_color = 8,
                KeyCode::Key9 => self.active_color = 9,
                KeyCode::Key0 => self.active_color = 10,
                KeyCode::Left => {
                    if self.active_tile > 0 {
                        self.active_tile -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.active_tile + 1 < self.doc.tile_count() {
                        self.active_tile += 1;
                    }
                }
                KeyCode::S => {
                    if state.is_key_down(KeyCode::LCtrl) || state.is_key_down(KeyCode::RCtrl) {
                        let path = std::path::Path::new("untitled.tiles");
                        if let Err(e) = crate::io::save_tiles(&self.doc, path) {
                            eprintln!("Save failed: {e}");
                        }
                    }
                }
                _ => {}
            },
            KeyState::Released => {
                if event.key == KeyCode::Space {
                    self.space_held = false;
                }
            }
        }
    }

    fn on_mouse(&mut self, _state: &mut State, event: MouseEvent) {
        match event.action {
            MouseAction::Pressed(MouseButton::Left) => {
                if self.space_held {
                    self.panning = true;
                    self.pan_anchor = event.world_pos;
                } else {
                    if let Some(idx) = self.palette_hit(event.world_pos) {
                        self.active_color = idx;
                        return;
                    }

                    if let Some((x, y)) = self.world_to_canvas(event.world_pos) {
                        self.history.save(&self.doc);
                        self.painting = true;
                        self.last_paint_pos = Some((x, y));
                        self.apply_tool_at(x, y);
                    }
                }
            }
            MouseAction::Released(MouseButton::Left) => {
                if self.panning {
                    self.panning = false;
                } else if self.painting {
                    if let Some((x, y)) = self.world_to_canvas(event.world_pos) {
                        self.finish_drag(x, y);
                    }
                    self.painting = false;
                    self.drag_start = None;
                    self.last_paint_pos = None;
                }
            }
            MouseAction::Scrolled(delta) => {
                let new_scale = (self.canvas_scale as i32 + delta.signum() as i32).clamp(1, 8) as u32;
                if new_scale != self.canvas_scale {
                    let old_scale = self.canvas_scale as f32;
                    let new_scale_f = new_scale as f32;

                    let local_x = event.world_pos.x - self.canvas_panel.pos.x;
                    let local_y = event.world_pos.y - self.canvas_panel.pos.y;

                    self.canvas_panel.pos.x += local_x * (1.0 - new_scale_f / old_scale);
                    self.canvas_panel.pos.y += local_y * (1.0 - new_scale_f / old_scale);

                    self.canvas_scale = new_scale;
                }
            }
            _ => {}
        }
    }
}
