use std::collections::HashMap;
use glam::Vec2;
use tiles::{Cell, KeyEvent, MouseAction, MouseButton, MouseEvent, State};
use tiles::font::Font;

use crate::config::{PaneConfig, PaneStyle};
use crate::layout::{Cursor, Rect};
use crate::widget::Widget;
use crate::widgets;

struct PaneState {
    position: Vec2,
    size: Vec2,
    _open: bool,
    movable: bool,
    resizable: bool,
    has_title_bar: bool,
    last_frame_elements: Vec<ElementEntry>,
}

struct ElementEntry {
    rect: Rect,
    cells: Vec<Cell>,
    is_separator: bool,
}

struct ActivePane {
    id: String,
    config: PaneConfig,
    style: PaneStyle,
    cursor: Cursor,
    elements: Vec<ElementEntry>,
    _content_origin: Vec2,
    title_bar_height: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ElementId {
    pane_index: usize,
    element_index: usize,
}

pub struct PaneContext {
    default_style: PaneStyle,
    viewport_size: Vec2,
    // Pane state persisted across frames
    pane_states: HashMap<String, PaneState>,
    // Focus and draw order
    focus_order: Vec<String>,
    // Current frame
    active_pane: Option<ActivePane>,
    completed_panes: Vec<CompletedPane>,
    // Input
    mouse_pos: Vec2,
    mouse_pressed: bool,
    mouse_released: bool,
    mouse_down: bool,
    mouse_press_pos: Option<Vec2>,
    // Mechanical interaction state
    active_element: Option<ElementId>,
    dragging_pane: Option<String>,
    drag_offset: Vec2,
    resizing_pane: Option<String>,
    resize_start_size: Vec2,
    resize_start_pos: Vec2,
}

struct CompletedPane {
    id: String,
    elements: Vec<ElementEntry>,
    bounds: Rect,
    title_bar_rect: Option<Rect>,
}

impl PaneContext {
    pub fn new() -> Self {
        Self {
            default_style: PaneStyle::default(),
            viewport_size: Vec2::ZERO,
            pane_states: HashMap::new(),
            focus_order: Vec::new(),
            active_pane: None,
            completed_panes: Vec::new(),
            mouse_pos: Vec2::ZERO,
            mouse_pressed: false,
            mouse_released: false,
            mouse_down: false,
            mouse_press_pos: None,
            active_element: None,
            dragging_pane: None,
            drag_offset: Vec2::ZERO,
            resizing_pane: None,
            resize_start_size: Vec2::ZERO,
            resize_start_pos: Vec2::ZERO,
        }
    }

    pub fn with_style(mut self, style: PaneStyle) -> Self {
        self.default_style = style;
        self
    }

    pub fn set_default_style(&mut self, style: PaneStyle) {
        self.default_style = style;
    }

    pub fn default_style(&self) -> &PaneStyle {
        &self.default_style
    }

    // --- Input feeding ---

    pub fn feed_mouse(&mut self, event: &MouseEvent) -> bool {
        self.mouse_pos = event.viewport_pos;

        match event.action {
            MouseAction::Pressed(MouseButton::Left) => {
                self.mouse_pressed = true;
                self.mouse_down = true;
                self.mouse_press_pos = Some(event.viewport_pos);

                // Check if click is on a pane title bar (for drag)
                if let Some(pane_id) = self.pane_at(event.viewport_pos) {
                    self.bring_to_front(&pane_id);

                    if let Some(state) = self.pane_states.get(&pane_id) {
                        let title_bar = self.title_bar_rect_for(&pane_id, state);
                        if let Some(tb) = title_bar {
                            if tb.contains(event.viewport_pos) {
                                if self.is_movable(&pane_id) {
                                    self.dragging_pane = Some(pane_id.clone());
                                    self.drag_offset = event.viewport_pos - state.position;
                                }
                                return true;
                            }
                        }

                        // Check resize handle (bottom-right corner)
                        if self.is_resizable(&pane_id) {
                            let bounds = Rect::new(
                                state.position.x,
                                state.position.y,
                                state.size.x,
                                state.size.y,
                            );
                            let handle = Rect::new(
                                bounds.x + bounds.w - 2.0,
                                bounds.y + bounds.h - 2.0,
                                2.0,
                                2.0,
                            );
                            if handle.contains(event.viewport_pos) {
                                self.resizing_pane = Some(pane_id.clone());
                                self.resize_start_size = state.size;
                                self.resize_start_pos = event.viewport_pos;
                                return true;
                            }
                        }
                    }

                    return true;
                }
                false
            }
            MouseAction::Released(MouseButton::Left) => {
                self.mouse_released = true;
                self.mouse_down = false;
                self.dragging_pane = None;
                self.resizing_pane = None;
                self.active_element = None;
                self.pane_at(event.viewport_pos).is_some()
            }
            MouseAction::Moved { .. } => {
                if let Some(ref pane_id) = self.dragging_pane.clone() {
                    if let Some(state) = self.pane_states.get_mut(pane_id) {
                        state.position = event.viewport_pos - self.drag_offset;
                        self.clamp_pane_to_viewport(pane_id);
                    }
                    return true;
                }
                if let Some(ref pane_id) = self.resizing_pane.clone() {
                    let delta = event.viewport_pos - self.resize_start_pos;
                    if let Some(state) = self.pane_states.get_mut(pane_id) {
                        let new_w = (self.resize_start_size.x + delta.x).max(10.0);
                        let new_h = (self.resize_start_size.y + delta.y).max(10.0);
                        state.size = Vec2::new(new_w, new_h);
                    }
                    return true;
                }
                self.pane_at(event.viewport_pos).is_some()
            }
            _ => false,
        }
    }

    pub fn feed_key(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    // --- Frame lifecycle ---

    pub fn start_frame(&mut self, viewport_size: Vec2) {
        self.viewport_size = viewport_size;
        self.completed_panes.clear();
        // mouse_pressed / mouse_released are NOT cleared here —
        // they are set by feed_mouse between frames and consumed during this frame.
        // They get cleared at the end of the frame via end_frame().
    }

    pub fn end_frame(&mut self) {
        self.mouse_pressed = false;
        self.mouse_released = false;
        self.mouse_press_pos = None;
    }

    // --- Pane API ---

    pub fn begin(&mut self, id: &str, config: PaneConfig) {
        debug_assert!(
            self.active_pane.is_none(),
            "panes: begin() called while another pane is open. Call end() first."
        );

        let style = config.style.clone().unwrap_or_else(|| self.default_style.clone());

        // Initialize or update pane state
        let has_title_bar = config.movable || config.title.is_some();
        if !self.pane_states.contains_key(id) {
            let pos = config.position.map(|(x, y)| Vec2::new(x, y)).unwrap_or_else(|| {
                Vec2::new(self.viewport_size.x / 2.0 - 30.0, self.viewport_size.y / 2.0 - 20.0)
            });
            let size = config.size.map(|(w, h)| Vec2::new(w, h)).unwrap_or(Vec2::ZERO);
            self.pane_states.insert(id.to_string(), PaneState {
                position: pos,
                size,
                _open: true,
                movable: config.movable,
                resizable: config.resizable,
                has_title_bar,
                last_frame_elements: Vec::new(),
            });
            self.focus_order.push(id.to_string());
        } else {
            let state = self.pane_states.get_mut(id).unwrap();
            state.movable = config.movable;
            state.resizable = config.resizable;
            state.has_title_bar = has_title_bar;
        }

        let state = self.pane_states.get(id).unwrap();
        let position = state.position;

        // Title bar height
        let title_bar_height = if config.movable || config.title.is_some() {
            style.font.height as f32 + style.padding * 2.0
        } else {
            0.0
        };

        let content_origin = Vec2::new(
            position.x + style.padding,
            position.y + title_bar_height + style.padding,
        );

        let cursor = Cursor::new(content_origin, style.element_spacing);

        self.active_pane = Some(ActivePane {
            id: id.to_string(),
            config,
            style,
            cursor,
            elements: Vec::new(),
            _content_origin: content_origin,
            title_bar_height,
        });
    }

    pub fn end(&mut self) {
        let mut pane = self.active_pane.take()
            .expect("panes: end() called without a matching begin()");

        let style = &pane.style;
        let state = self.pane_states.get_mut(&pane.id).unwrap();

        // Compute final size
        let content_size = pane.cursor.content_size();
        let total_w;
        let total_h;

        if let Some((w, h)) = pane.config.size {
            total_w = w;
            total_h = h;
        } else {
            total_w = content_size.x + style.padding * 2.0;
            total_h = content_size.y + style.padding * 2.0 + pane.title_bar_height;
        }

        // Regenerate separator cells to span content width
        let content_w = total_w - style.padding * 2.0;
        for elem in &mut pane.elements {
            if elem.is_separator {
                elem.cells = widgets::SeparatorWidget::render_at_width(elem.rect, content_w, style);
            }
        }

        state.size = Vec2::new(total_w, total_h);
        state.last_frame_elements = pane.elements.iter().map(|e| ElementEntry {
            rect: e.rect,
            cells: Vec::new(),
            is_separator: false,
        }).collect();

        let bounds = Rect::new(state.position.x, state.position.y, total_w, total_h);

        let title_bar_rect = if pane.title_bar_height > 0.0 {
            Some(Rect::new(state.position.x, state.position.y, total_w, pane.title_bar_height))
        } else {
            None
        };

        self.completed_panes.push(CompletedPane {
            id: pane.id.clone(),
            elements: pane.elements,
            bounds,
            title_bar_rect,
        });
    }

    // --- Widget convenience methods ---

    pub fn button(&mut self, label: &str) -> bool {
        let pane = self.active_pane.as_mut().expect("panes: button() called outside begin()/end()");
        let style = pane.style.clone();
        let widget = widgets::ButtonWidget { label: label.to_string() };
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);
        let elem_idx = pane.elements.len();

        let hovered = rect.contains(self.mouse_pos);
        let clicked = hovered && self.mouse_pressed;
        let active = hovered && self.mouse_down;

        let cells = widget.render(rect, &style, hovered, active);
        pane.elements.push(ElementEntry { rect, cells, is_separator: false });

        if clicked {
            self.active_element = Some(ElementId {
                pane_index: self.completed_panes.len(),
                element_index: elem_idx,
            });
        }

        clicked
    }

    pub fn text(&mut self, content: &str) {
        let pane = self.active_pane.as_mut().expect("panes: text() called outside begin()/end()");
        let style = pane.style.clone();
        let widget = widgets::TextWidget { content: content.to_string() };
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);
        let cells = widget.render(rect, &style, false, false);
        pane.elements.push(ElementEntry { rect, cells, is_separator: false });
    }

    pub fn slider(&mut self, label: &str, current: f32, min: f32, max: f32) -> Option<f32> {
        let pane = self.active_pane.as_mut().expect("panes: slider() called outside begin()/end()");
        let style = pane.style.clone();
        let widget = widgets::SliderWidget {
            label: label.to_string(),
            value: current,
            min,
            max,
        };
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);
        let elem_idx = pane.elements.len();

        let hovered = rect.contains(self.mouse_pos);
        let active = hovered && self.mouse_down;

        let cells = widget.render(rect, &style, hovered, active);
        pane.elements.push(ElementEntry { rect, cells, is_separator: false });

        // Slider interaction: click/drag sets value based on X position within track
        let track_x = rect.x + widget.label_width(&style) + 1.0;
        let track_w = widget.track_width(&style);

        let interacting = if let Some(ref ae) = self.active_element {
            ae.pane_index == self.completed_panes.len() && ae.element_index == elem_idx
        } else {
            false
        };

        if (hovered && self.mouse_pressed) || interacting {
            if self.mouse_pressed && hovered {
                self.active_element = Some(ElementId {
                    pane_index: self.completed_panes.len(),
                    element_index: elem_idx,
                });
            }
            let t = ((self.mouse_pos.x - track_x) / track_w).clamp(0.0, 1.0);
            let new_val = min + t * (max - min);
            if (new_val - current).abs() > f32::EPSILON {
                return Some(new_val);
            }
        }

        None
    }

    pub fn checkbox(&mut self, label: &str, current: bool) -> Option<bool> {
        let pane = self.active_pane.as_mut().expect("panes: checkbox() called outside begin()/end()");
        let style = pane.style.clone();
        let widget = widgets::CheckboxWidget {
            label: label.to_string(),
            checked: current,
        };
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);

        let hovered = rect.contains(self.mouse_pos);
        let clicked = hovered && self.mouse_pressed;

        let cells = widget.render(rect, &style, hovered, false);
        pane.elements.push(ElementEntry { rect, cells, is_separator: false });

        if clicked {
            Some(!current)
        } else {
            None
        }
    }

    pub fn separator(&mut self) {
        let pane = self.active_pane.as_mut().expect("panes: separator() called outside begin()/end()");
        let style = pane.style.clone();
        let widget = widgets::SeparatorWidget;
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);
        let cells = widget.render(rect, &style, false, false);
        pane.elements.push(ElementEntry { rect, cells, is_separator: true });
    }

    pub fn spacer(&mut self, height: f32) {
        let pane = self.active_pane.as_mut().expect("panes: spacer() called outside begin()/end()");
        pane.cursor.advance_vertical(height);
    }

    pub fn spacer_h(&mut self, width: f32) {
        let pane = self.active_pane.as_mut().expect("panes: spacer_h() called outside begin()/end()");
        pane.cursor.advance_horizontal(width);
    }

    pub fn same_line(&mut self) {
        let pane = self.active_pane.as_mut().expect("panes: same_line() called outside begin()/end()");
        pane.cursor.set_same_line();
    }

    pub fn add<W: Widget>(&mut self, widget: W) -> Rect {
        let pane = self.active_pane.as_mut().expect("panes: add() called outside begin()/end()");
        let style = pane.style.clone();
        let (w, h) = widget.size(&style);
        let rect = pane.cursor.allocate(w, h);
        let hovered = rect.contains(self.mouse_pos);
        let active = hovered && self.mouse_down;
        let cells = widget.render(rect, &style, hovered, active);
        pane.elements.push(ElementEntry { rect, cells, is_separator: false });
        rect
    }

    pub fn hit_test(&self, rect: Rect) -> HitState {
        let hovered = rect.contains(self.mouse_pos);
        HitState {
            hovered,
            pressed: hovered && self.mouse_pressed,
            held: hovered && self.mouse_down,
            released: hovered && self.mouse_released,
        }
    }

    // --- Rendering ---

    pub fn render_all(&self, state: &mut State) {
        for pane_id in &self.focus_order {
            if let Some(completed) = self.completed_panes.iter().find(|p| &p.id == pane_id) {
                let _pane_state = self.pane_states.get(pane_id).unwrap();
                let style = self.style_for(pane_id);
                let bounds = completed.bounds;

                // Render pane background
                self.render_pane_background(state, &bounds, &style);

                // Render title bar
                if let Some(tb) = completed.title_bar_rect {
                    self.render_title_bar(state, pane_id, &tb, &style);
                }

                // Render elements (clipped to pane bounds)
                for elem in &completed.elements {
                    for cell in &elem.cells {
                        let pos = Vec2::new(cell.position.x, cell.position.y);
                        if bounds.contains(pos) {
                            state.draw_screen(*cell);
                        }
                    }
                }

                // Render border
                if style.border {
                    self.render_border(state, &bounds, &style);
                }
            }
        }
    }

    // --- Internal helpers ---

    fn render_pane_background(&self, state: &mut State, bounds: &Rect, style: &PaneStyle) {
        let cr = style.corner_radius as f32;
        for y in 0..bounds.h as u32 {
            for x in 0..bounds.w as u32 {
                if self.is_corner_clipped(x as f32, y as f32, bounds.w, bounds.h, cr) {
                    continue;
                }
                state.draw_screen(
                    Cell::new(bounds.x + x as f32, bounds.y + y as f32)
                        .rgba(style.background_color[0], style.background_color[1],
                              style.background_color[2], style.background_color[3])
                );
            }
        }
    }

    fn render_title_bar(&self, state: &mut State, pane_id: &str, tb: &Rect, style: &PaneStyle) {
        let cr = style.corner_radius as f32;
        for y in 0..tb.h as u32 {
            for x in 0..tb.w as u32 {
                if y == 0 && self.is_top_corner_clipped(x as f32, tb.w, cr) {
                    continue;
                }
                state.draw_screen(
                    Cell::new(tb.x + x as f32, tb.y + y as f32)
                        .rgba(style.title_background_color[0], style.title_background_color[1],
                              style.title_background_color[2], style.title_background_color[3])
                );
            }
        }

        // Render title text
        let title = self.title_for(pane_id);
        let text_x = tb.x + style.padding;
        let text_y = tb.y + style.padding;
        self.render_text(state, &title, style.font, text_x, text_y, style.title_text_color);
    }

    fn render_border(&self, state: &mut State, bounds: &Rect, style: &PaneStyle) {
        let w = bounds.w as u32;
        let h = bounds.h as u32;
        let cr = style.corner_radius as u32;
        let color = style.border_color;

        // Top and bottom edges
        for x in cr..w.saturating_sub(cr) {
            state.draw_screen(
                Cell::new(bounds.x + x as f32, bounds.y).rgba(color[0], color[1], color[2], color[3])
            );
            state.draw_screen(
                Cell::new(bounds.x + x as f32, bounds.y + (h - 1) as f32).rgba(color[0], color[1], color[2], color[3])
            );
        }

        // Left and right edges
        for y in cr..h.saturating_sub(cr) {
            state.draw_screen(
                Cell::new(bounds.x, bounds.y + y as f32).rgba(color[0], color[1], color[2], color[3])
            );
            state.draw_screen(
                Cell::new(bounds.x + (w - 1) as f32, bounds.y + y as f32).rgba(color[0], color[1], color[2], color[3])
            );
        }
    }

    fn render_text(&self, state: &mut State, text: &str, font: &Font, x: f32, y: f32, color: [f32; 4]) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(glyph) = font.glyph(ch) {
                for row in 0..glyph.height as usize {
                    for col in 0..glyph.width as usize {
                        if glyph.pixel(col, row) {
                            state.draw_screen(
                                Cell::new(cursor_x + col as f32, y + glyph.top as f32 + row as f32)
                                    .rgba(color[0], color[1], color[2], color[3])
                            );
                        }
                    }
                }
            }
            cursor_x += font.char_advance(ch) as f32;
        }
    }

    fn is_corner_clipped(&self, x: f32, y: f32, w: f32, h: f32, radius: f32) -> bool {
        if radius == 0.0 {
            return false;
        }
        // Top-left
        if x < radius && y < radius {
            return x + y < radius;
        }
        // Top-right
        if x >= w - radius && y < radius {
            return (w - 1.0 - x) + y < radius;
        }
        // Bottom-left
        if x < radius && y >= h - radius {
            return x + (h - 1.0 - y) < radius;
        }
        // Bottom-right
        if x >= w - radius && y >= h - radius {
            return (w - 1.0 - x) + (h - 1.0 - y) < radius;
        }
        false
    }

    fn is_top_corner_clipped(&self, x: f32, w: f32, radius: f32) -> bool {
        if radius == 0.0 {
            return false;
        }
        x < radius || x >= w - radius
    }

    fn pane_at(&self, pos: Vec2) -> Option<String> {
        // Check in reverse focus order (front-most first)
        for id in self.focus_order.iter().rev() {
            if let Some(state) = self.pane_states.get(id) {
                let bounds = Rect::new(state.position.x, state.position.y, state.size.x, state.size.y);
                if bounds.contains(pos) {
                    return Some(id.clone());
                }
            }
        }
        None
    }

    fn bring_to_front(&mut self, id: &str) {
        if let Some(pos) = self.focus_order.iter().position(|x| x == id) {
            let removed = self.focus_order.remove(pos);
            self.focus_order.push(removed);
        }
    }

    fn clamp_pane_to_viewport(&mut self, id: &str) {
        if let Some(state) = self.pane_states.get_mut(id) {
            // Keep title bar visible (at least top 5 cells within viewport)
            let min_visible = 5.0_f32.min(state.size.y);
            state.position.x = state.position.x.clamp(
                -(state.size.x - min_visible),
                self.viewport_size.x - min_visible,
            );
            state.position.y = state.position.y.clamp(0.0, self.viewport_size.y - min_visible);
        }
    }

    fn title_bar_rect_for(&self, _id: &str, state: &PaneState) -> Option<Rect> {
        if state.has_title_bar && state.size.x > 0.0 && state.size.y > 0.0 {
            let style = &self.default_style;
            let tb_height = style.font.height as f32 + style.padding * 2.0;
            return Some(Rect::new(state.position.x, state.position.y, state.size.x, tb_height));
        }
        None
    }

    fn is_movable(&self, id: &str) -> bool {
        self.pane_states.get(id).map(|s| s.movable).unwrap_or(false)
    }

    fn is_resizable(&self, id: &str) -> bool {
        self.pane_states.get(id).map(|s| s.resizable).unwrap_or(false)
    }

    fn style_for(&self, _id: &str) -> PaneStyle {
        self.default_style.clone()
    }

    fn title_for(&self, pane_id: &str) -> String {
        if let Some(completed) = self.completed_panes.iter().find(|p| p.id == pane_id) {
            // We don't store the config in completed panes, use pane_id as fallback
            let _ = completed;
        }
        pane_id.to_string()
    }
}

impl Default for PaneContext {
    fn default() -> Self {
        Self::new()
    }
}

pub struct HitState {
    pub hovered: bool,
    pub pressed: bool,
    pub held: bool,
    pub released: bool,
}
