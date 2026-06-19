use crate::cell::Cell;
use crate::color::Color;
use crate::element::{test_shape, DragInfo};
use crate::input::{InputState, MouseButton};
use crate::rect::Rect;
use crate::runner::{App, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    Flow,
    Relative(i32, i32),
    Absolute(i32, i32),
}

#[derive(Clone, Debug)]
pub struct Style {
    pub w: Option<i32>,
    pub h: Option<i32>,
    pub fill_w: bool,
    pub fill_h: bool,
    pub color: Option<Color>,
    pub hover_color: Option<Color>,
    pub pressed_color: Option<Color>,
    pub axis: Option<Axis>,
    pub gap: Option<i32>,
    pub padding: Option<i32>,
    pub position: Position,
    pub z_index: i32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            w: None,
            h: None,
            fill_w: false,
            fill_h: false,
            color: None,
            hover_color: None,
            pressed_color: None,
            axis: None,
            gap: None,
            padding: None,
            position: Position::Flow,
            z_index: 0,
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn w(mut self, w: i32) -> Self {
        self.w = Some(w);
        self
    }

    pub fn h(mut self, h: i32) -> Self {
        self.h = Some(h);
        self
    }

    pub fn size(mut self, w: i32, h: i32) -> Self {
        self.w = Some(w);
        self.h = Some(h);
        self
    }

    pub fn fill_w(mut self) -> Self {
        self.fill_w = true;
        self
    }

    pub fn fill_h(mut self) -> Self {
        self.fill_h = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.hover_color = Some(color);
        self
    }

    pub fn pressed_color(mut self, color: Color) -> Self {
        self.pressed_color = Some(color);
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = Some(axis);
        self
    }

    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn padding(mut self, padding: i32) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }

    fn apply_from(&mut self, other: &Style) {
        if other.w.is_some() { self.w = other.w; }
        if other.h.is_some() { self.h = other.h; }
        if other.fill_w { self.fill_w = true; }
        if other.fill_h { self.fill_h = true; }
        if other.color.is_some() { self.color = other.color; }
        if other.hover_color.is_some() { self.hover_color = other.hover_color; }
        if other.pressed_color.is_some() { self.pressed_color = other.pressed_color; }
        if other.axis.is_some() { self.axis = other.axis; }
        if other.gap.is_some() { self.gap = other.gap; }
        if other.padding.is_some() { self.padding = other.padding; }
        if other.position != Position::Flow { self.position = other.position; }
        if other.z_index != 0 { self.z_index = other.z_index; }
    }

    fn inherit_from(&mut self, parent: &Style) {
        if self.color.is_none() { self.color = parent.color; }
        if self.hover_color.is_none() { self.hover_color = parent.hover_color; }
        if self.pressed_color.is_none() { self.pressed_color = parent.pressed_color; }
        if self.axis.is_none() { self.axis = parent.axis; }
        if self.gap.is_none() { self.gap = parent.gap; }
    }
}

pub struct Handlers<A: App> {
    pub on_hover: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_enter: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_leave: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_click: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_double_click: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_press: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_release: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_right_click: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_hold: Option<Box<dyn FnMut(&mut A, &mut State)>>,
    pub on_drag: Option<Box<dyn FnMut(&mut A, &mut State, DragInfo)>>,
    pub on_drag_end: Option<Box<dyn FnMut(&mut A, &mut State, DragInfo)>>,
    pub on_scroll: Option<Box<dyn FnMut(&mut A, &mut State, f32)>>,
}

impl<A: App> Default for Handlers<A> {
    fn default() -> Self {
        Self {
            on_hover: None,
            on_enter: None,
            on_leave: None,
            on_click: None,
            on_double_click: None,
            on_press: None,
            on_release: None,
            on_right_click: None,
            on_hold: None,
            on_drag: None,
            on_drag_end: None,
            on_scroll: None,
        }
    }
}

impl<A: App> Handlers<A> {
    fn has_action_handler(&self) -> bool {
        self.on_click.is_some()
            || self.on_double_click.is_some()
            || self.on_press.is_some()
            || self.on_release.is_some()
            || self.on_right_click.is_some()
            || self.on_hold.is_some()
            || self.on_drag.is_some()
            || self.on_drag_end.is_some()
            || self.on_scroll.is_some()
    }
}

pub struct Node<A: App> {
    style: Style,
    children: Vec<Node<A>>,
    handlers: Handlers<A>,
}

impl<A: App> Node<A> {
    pub fn new() -> Self {
        Self {
            style: Style::default(),
            children: Vec::new(),
            handlers: Handlers::default(),
        }
    }

    // --- Style builder methods ---

    pub fn w(mut self, w: i32) -> Self {
        self.style.w = Some(w);
        self
    }

    pub fn h(mut self, h: i32) -> Self {
        self.style.h = Some(h);
        self
    }

    pub fn size(mut self, w: i32, h: i32) -> Self {
        self.style.w = Some(w);
        self.style.h = Some(h);
        self
    }

    pub fn fill_w(mut self) -> Self {
        self.style.fill_w = true;
        self
    }

    pub fn fill_h(mut self) -> Self {
        self.style.fill_h = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }

    pub fn hover_color(mut self, color: Color) -> Self {
        self.style.hover_color = Some(color);
        self
    }

    pub fn pressed_color(mut self, color: Color) -> Self {
        self.style.pressed_color = Some(color);
        self
    }

    pub fn axis(mut self, axis: Axis) -> Self {
        self.style.axis = Some(axis);
        self
    }

    pub fn gap(mut self, gap: i32) -> Self {
        self.style.gap = Some(gap);
        self
    }

    pub fn padding(mut self, padding: i32) -> Self {
        self.style.padding = Some(padding);
        self
    }

    pub fn relative(mut self, x: i32, y: i32) -> Self {
        self.style.position = Position::Relative(x, y);
        self
    }

    pub fn absolute(mut self, x: i32, y: i32) -> Self {
        self.style.position = Position::Absolute(x, y);
        self
    }

    pub fn z_index(mut self, z: i32) -> Self {
        self.style.z_index = z;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style.apply_from(&style);
        self
    }

    // --- Children ---

    pub fn children(mut self, children: Vec<Node<A>>) -> Self {
        self.children = children;
        self
    }

    // --- Handler builder methods ---

    pub fn on_hover(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_hover = Some(Box::new(f));
        self
    }

    pub fn on_enter(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_enter = Some(Box::new(f));
        self
    }

    pub fn on_leave(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_leave = Some(Box::new(f));
        self
    }

    pub fn on_click(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_click = Some(Box::new(f));
        self
    }

    pub fn on_double_click(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_double_click = Some(Box::new(f));
        self
    }

    pub fn on_press(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_press = Some(Box::new(f));
        self
    }

    pub fn on_release(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_release = Some(Box::new(f));
        self
    }

    pub fn on_right_click(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_right_click = Some(Box::new(f));
        self
    }

    pub fn on_hold(mut self, f: impl FnMut(&mut A, &mut State) + 'static) -> Self {
        self.handlers.on_hold = Some(Box::new(f));
        self
    }

    pub fn on_drag(mut self, f: impl FnMut(&mut A, &mut State, DragInfo) + 'static) -> Self {
        self.handlers.on_drag = Some(Box::new(f));
        self
    }

    pub fn on_drag_end(mut self, f: impl FnMut(&mut A, &mut State, DragInfo) + 'static) -> Self {
        self.handlers.on_drag_end = Some(Box::new(f));
        self
    }

    pub fn on_scroll(mut self, f: impl FnMut(&mut A, &mut State, f32) + 'static) -> Self {
        self.handlers.on_scroll = Some(Box::new(f));
        self
    }
}

// --- Convenience constructors ---

pub fn button<A: App>() -> Node<A> {
    Node::new()
}

pub fn row<A: App>() -> Node<A> {
    Node::new().axis(Axis::Row)
}

pub fn col<A: App>() -> Node<A> {
    Node::new().axis(Axis::Column)
}

// --- Layout ---

struct LayoutCtx {
    origin_x: i32,
    origin_y: i32,
    available_w: i32,
    available_h: i32,
}

struct ResolvedNode {
    rect: Rect,
    style: Style,
    children: Vec<ResolvedNode>,
    handler_idx: usize,
    is_positioned: bool,
}

fn layout_node<A: App>(
    node: &Node<A>,
    ctx: &LayoutCtx,
    parent_style: &Style,
    handler_idx: &mut usize,
) -> ResolvedNode {
    let mut style = node.style.clone();
    style.inherit_from(parent_style);

    let padding = style.padding.unwrap_or(0);
    let axis = style.axis.unwrap_or(Axis::Column);
    let gap = style.gap.unwrap_or(0);

    let my_handler_idx = *handler_idx;
    *handler_idx += 1;

    let is_positioned = style.position != Position::Flow;

    // Resolve own dimensions (before children, for fill)
    let own_w = if style.fill_w {
        Some(ctx.available_w)
    } else {
        style.w
    };
    let own_h = if style.fill_h {
        Some(ctx.available_h)
    } else {
        style.h
    };

    // Determine origin based on positioning
    let (origin_x, origin_y) = match style.position {
        Position::Flow => (ctx.origin_x, ctx.origin_y),
        Position::Relative(rx, ry) => (ctx.origin_x + rx, ctx.origin_y + ry),
        Position::Absolute(ax, ay) => (ax, ay),
    };

    // Available space for children
    let content_available_w = own_w.unwrap_or(ctx.available_w) - padding * 2;
    let content_available_h = own_h.unwrap_or(ctx.available_h) - padding * 2;

    let content_origin_x = origin_x + padding;
    let content_origin_y = origin_y + padding;

    // Layout children
    let mut resolved_children = Vec::with_capacity(node.children.len());
    let mut cursor_x = content_origin_x;
    let mut cursor_y = content_origin_y;
    let mut max_cross: i32 = 0;
    let mut child_count = 0;

    for child in &node.children {
        let child_ctx = LayoutCtx {
            origin_x: cursor_x,
            origin_y: cursor_y,
            available_w: content_available_w,
            available_h: content_available_h,
        };

        let resolved_child = layout_node(child, &child_ctx, &style, handler_idx);

        if resolved_child.is_positioned {
            resolved_children.push(resolved_child);
            continue;
        }

        let child_w = resolved_child.rect.width() as i32;
        let child_h = resolved_child.rect.height() as i32;

        match axis {
            Axis::Row => {
                cursor_x += child_w + gap;
                max_cross = max_cross.max(child_h);
            }
            Axis::Column => {
                cursor_y += child_h + gap;
                max_cross = max_cross.max(child_w);
            }
        }
        child_count += 1;

        resolved_children.push(resolved_child);
    }

    // Compute content size
    let (content_w, content_h) = match axis {
        Axis::Row => {
            let main = (cursor_x - content_origin_x) - if child_count > 0 { gap } else { 0 };
            (main.max(0), max_cross)
        }
        Axis::Column => {
            let main = (cursor_y - content_origin_y) - if child_count > 0 { gap } else { 0 };
            (max_cross, main.max(0))
        }
    };

    let final_w = own_w.unwrap_or(content_w + padding * 2);
    let final_h = own_h.unwrap_or(content_h + padding * 2);

    let rect = Rect::from_top_left(
        origin_x as f32,
        origin_y as f32,
        final_w.max(0) as u32,
        final_h.max(0) as u32,
    );

    ResolvedNode {
        rect,
        style,
        children: resolved_children,
        handler_idx: my_handler_idx,
        is_positioned,
    }
}

// --- Evaluate (hit-test + handlers + draw) ---

struct ConsumedState {
    left: bool,
    right: bool,
    middle: bool,
    scroll: bool,
}

impl ConsumedState {
    fn new() -> Self {
        Self {
            left: false,
            right: false,
            middle: false,
            scroll: false,
        }
    }
}

fn collect_handlers<A: App>(node: Node<A>, out: &mut Vec<Handlers<A>>) {
    out.push(node.handlers);
    for child in node.children {
        collect_handlers(child, out);
    }
}

fn evaluate_node<A: App>(
    resolved: &ResolvedNode,
    handlers: &mut Vec<Handlers<A>>,
    app: &mut A,
    state: &mut State,
    input: &InputState,
    cells: &mut Vec<Cell>,
    consumed: &mut ConsumedState,
    depth: f32,
) {
    // First: process positioned children sorted by z_index (highest first for hit-test)
    let mut positioned_indices: Vec<usize> = Vec::new();
    let mut flow_indices: Vec<usize> = Vec::new();

    for (i, child) in resolved.children.iter().enumerate() {
        if child.is_positioned {
            positioned_indices.push(i);
        } else {
            flow_indices.push(i);
        }
    }

    // Sort positioned by z_index descending (highest z tested first)
    positioned_indices.sort_by(|a, b| {
        resolved.children[*b]
            .style
            .z_index
            .cmp(&resolved.children[*a].style.z_index)
    });

    // Hit-test positioned children first (they're on top)
    for &i in &positioned_indices {
        evaluate_node(
            &resolved.children[i],
            handlers,
            app,
            state,
            input,
            cells,
            consumed,
            depth + 1.0,
        );
    }

    // Hit-test flow children (in reverse order — later siblings are visually on top)
    for &i in flow_indices.iter().rev() {
        evaluate_node(
            &resolved.children[i],
            handlers,
            app,
            state,
            input,
            cells,
            consumed,
            depth + 1.0,
        );
    }

    // Hit-test this node
    let hit = test_shape(input, &resolved.rect, true);
    let handler = &mut handlers[resolved.handler_idx];

    // Spatial handlers always fire (propagate regardless of children)
    if hit.is_hovered() {
        if let Some(f) = &mut handler.on_hover {
            f(app, state);
        }
    }
    if hit.has_entered() {
        if let Some(f) = &mut handler.on_enter {
            f(app, state);
        }
    }
    if hit.has_left() {
        if let Some(f) = &mut handler.on_leave {
            f(app, state);
        }
    }

    // Action handlers: deepest-with-handler wins (children already had their chance)
    if !consumed.left && handler.has_action_handler() {
        if hit.is_clicked() {
            if let Some(f) = &mut handler.on_click {
                f(app, state);
                consumed.left = true;
            }
        }
        if hit.is_double_clicked() {
            if let Some(f) = &mut handler.on_double_click {
                f(app, state);
                consumed.left = true;
            }
        }
        if hit.is_pressed() {
            if let Some(f) = &mut handler.on_press {
                f(app, state);
                consumed.left = true;
            }
        }
        if hit.is_released() {
            if let Some(f) = &mut handler.on_release {
                f(app, state);
                consumed.left = true;
            }
        }
        if hit.is_held().is_some() {
            if let Some(f) = &mut handler.on_hold {
                f(app, state);
                consumed.left = true;
            }
        }
        if let Some(drag) = hit.is_dragging() {
            if let Some(f) = &mut handler.on_drag {
                f(app, state, drag);
                consumed.left = true;
            }
        }
        if let Some(drag) = hit.is_drag_end() {
            if let Some(f) = &mut handler.on_drag_end {
                f(app, state, drag);
                consumed.left = true;
            }
        }
    }

    if !consumed.right {
        if hit.is_right_clicked() {
            if let Some(f) = &mut handler.on_right_click {
                f(app, state);
                consumed.right = true;
            }
        }
    }

    if !consumed.scroll {
        if let Some(delta) = hit.is_scrolling() {
            if let Some(f) = &mut handler.on_scroll {
                f(app, state, delta);
                consumed.scroll = true;
            }
        }
    }

    // Draw: emit cells for this node (behind children via z-depth)
    let draw_color = if hit.is_down() {
        resolved.style.pressed_color.or(resolved.style.hover_color).or(resolved.style.color)
    } else if hit.is_hovered() {
        resolved.style.hover_color.or(resolved.style.color)
    } else {
        resolved.style.color
    };

    if let Some(color) = draw_color {
        let x0 = resolved.rect.x() as i32;
        let y0 = resolved.rect.y() as i32;
        let w = resolved.rect.width() as i32;
        let h = resolved.rect.height() as i32;
        let z = depth;

        for dy in 0..h {
            for dx in 0..w {
                cells.push(
                    Cell::new_3d((x0 + dx) as f32, (y0 + dy) as f32, z).color(color),
                );
            }
        }
    }

    // Draw flow children (emit on top of parent) — already evaluated above,
    // but cells were pushed during recursion so they're naturally on top
}

// --- Public API ---

pub struct UiResult {
    consumed: ConsumedState,
}

impl UiResult {
    pub fn consumed_by_ui(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.consumed.left,
            MouseButton::Right => self.consumed.right,
            MouseButton::Middle => self.consumed.middle,
        }
    }

    pub fn click_consumed_by_ui(&self) -> bool {
        self.consumed.left
    }

    pub fn right_click_consumed_by_ui(&self) -> bool {
        self.consumed.right
    }

    pub fn scroll_consumed_by_ui(&self) -> bool {
        self.consumed.scroll
    }

    pub fn any_consumed_by_ui(&self) -> bool {
        self.consumed.left || self.consumed.right || self.consumed.middle || self.consumed.scroll
    }
}

pub(crate) fn evaluate<A: App>(
    root: Node<A>,
    app: &mut A,
    state: &mut State,
    input: &InputState,
    screen_w: i32,
    screen_h: i32,
) -> (Vec<Cell>, UiResult) {
    let ctx = LayoutCtx {
        origin_x: 0,
        origin_y: 0,
        available_w: screen_w,
        available_h: screen_h,
    };

    let parent_style = Style::default();
    let mut handler_idx = 0;
    let resolved = layout_node(&root, &ctx, &parent_style, &mut handler_idx);

    let mut handlers = Vec::with_capacity(handler_idx);
    collect_handlers(root, &mut handlers);

    let mut cells = Vec::new();
    let mut consumed = ConsumedState::new();

    evaluate_node(
        &resolved,
        &mut handlers,
        app,
        state,
        input,
        &mut cells,
        &mut consumed,
        0.0,
    );

    (cells, UiResult { consumed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::input::{ButtonState, InputState, MouseButton};
    use glam::Vec2;

    const RED: Color = Color::linear(1.0, 0.0, 0.0, 1.0);
    const BLUE: Color = Color::linear(0.0, 0.0, 1.0, 1.0);
    const GREEN: Color = Color::linear(0.0, 1.0, 0.0, 1.0);
    const GREY: Color = Color::linear(0.3, 0.3, 0.3, 1.0);

    struct TestApp {
        clicked: bool,
        hovered: bool,
        count: i32,
        drag_delta: Vec2,
        scroll_amount: f32,
    }

    impl TestApp {
        fn new() -> Self {
            Self {
                clicked: false,
                hovered: false,
                count: 0,
                drag_delta: Vec2::ZERO,
                scroll_amount: 0.0,
            }
        }
    }

    impl App for TestApp {
        fn update(&mut self, _state: &mut State) {}
    }

    fn make_state() -> State {
        State::new_for_test(256, 256)
    }

    fn input_at(x: f32, y: f32) -> InputState {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(x, y);
        input.prev_mouse_screen_pos = Vec2::new(x, y);
        input.mouse_world_pos = Vec2::new(x, y);
        input.prev_mouse_world_pos = Vec2::new(x, y);
        input
    }

    fn input_with_click_at(x: f32, y: f32) -> InputState {
        let mut input = input_at(x, y);
        input.left_press_screen_pos = Vec2::new(x, y);
        input.left_press_world_pos = Vec2::new(x, y);
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.held_duration = 0.05;
        left.press_count = 1;
        input
    }

    // --- Layout tests ---

    #[test]
    fn empty_node_zero_size() {
        let node: Node<TestApp> = Node::new();
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 0);
        assert_eq!(resolved.rect.height(), 0);
    }

    #[test]
    fn explicit_size() {
        let node: Node<TestApp> = Node::new().size(10, 5);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 10);
        assert_eq!(resolved.rect.height(), 5);
    }

    #[test]
    fn fill_w_takes_available() {
        let node: Node<TestApp> = Node::new().fill_w().h(10);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 100,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 100);
        assert_eq!(resolved.rect.height(), 10);
    }

    #[test]
    fn column_layout_stacks_vertically() {
        let node: Node<TestApp> = col().children(vec![
            Node::new().size(10, 5),
            Node::new().size(10, 5),
            Node::new().size(10, 5),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 10);
        assert_eq!(resolved.rect.height(), 15);
        assert_eq!(resolved.children[0].rect.y(), 0.0);
        assert_eq!(resolved.children[1].rect.y(), 5.0);
        assert_eq!(resolved.children[2].rect.y(), 10.0);
    }

    #[test]
    fn row_layout_stacks_horizontally() {
        let node: Node<TestApp> = row().children(vec![
            Node::new().size(10, 5),
            Node::new().size(10, 5),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 20);
        assert_eq!(resolved.rect.height(), 5);
        assert_eq!(resolved.children[0].rect.x(), 0.0);
        assert_eq!(resolved.children[1].rect.x(), 10.0);
    }

    #[test]
    fn gap_between_children() {
        let node: Node<TestApp> = row().gap(4).children(vec![
            Node::new().size(10, 5),
            Node::new().size(10, 5),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 24);
        assert_eq!(resolved.children[1].rect.x(), 14.0);
    }

    #[test]
    fn padding_offsets_children() {
        let node: Node<TestApp> = col().padding(3).children(vec![
            Node::new().size(4, 4),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        assert_eq!(resolved.rect.width(), 10); // 4 + 3*2
        assert_eq!(resolved.rect.height(), 10);
        assert_eq!(resolved.children[0].rect.x(), 3.0);
        assert_eq!(resolved.children[0].rect.y(), 3.0);
    }

    #[test]
    fn nested_layout() {
        let node: Node<TestApp> = col().padding(2).children(vec![
            row().gap(2).children(vec![
                Node::new().size(5, 5),
                Node::new().size(5, 5),
            ]),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        // Inner row: 5+2+5=12 wide, 5 tall
        // Outer: 12+4=16 wide, 5+4=9 tall
        assert_eq!(resolved.rect.width(), 16);
        assert_eq!(resolved.rect.height(), 9);
    }

    #[test]
    fn absolute_position_skips_cursor() {
        let node: Node<TestApp> = col().children(vec![
            Node::new().size(10, 10),
            Node::new().size(5, 5).absolute(50, 50),
            Node::new().size(10, 10),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        // Absolute node doesn't affect parent size
        assert_eq!(resolved.rect.height(), 20); // only 2 flow children
        // Absolute node at (50, 50)
        let abs_child = &resolved.children[1];
        assert_eq!(abs_child.rect.x(), 50.0);
        assert_eq!(abs_child.rect.y(), 50.0);
        // Third child at y=10 (not y=15)
        assert_eq!(resolved.children[2].rect.y(), 10.0);
    }

    #[test]
    fn relative_position() {
        let node: Node<TestApp> = col().padding(5).children(vec![
            Node::new().size(10, 10).relative(3, 3),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        // Child positioned at parent cursor (5,5) + relative offset (3,3) = (8,8)
        let child = &resolved.children[0];
        assert_eq!(child.rect.x(), 8.0);
        assert_eq!(child.rect.y(), 8.0);
    }

    // --- Draw tests ---

    #[test]
    fn colored_node_emits_cells() {
        let node: Node<TestApp> = Node::new().size(3, 2).color(RED);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(cells.len(), 6);
        assert_eq!(cells[0].color, RED.to_array());
    }

    #[test]
    fn uncolored_node_emits_no_cells() {
        let node: Node<TestApp> = Node::new().size(3, 2);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(cells.len(), 0);
    }

    #[test]
    fn hover_color_on_hover() {
        let node: Node<TestApp> = Node::new().size(10, 10).color(RED).hover_color(BLUE);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(5.0, 5.0); // inside
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn normal_color_when_not_hovered() {
        let node: Node<TestApp> = Node::new().size(10, 10).color(RED).hover_color(BLUE);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(50.0, 50.0); // outside
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(cells[0].color, RED.to_array());
    }

    // --- Handler tests ---

    #[test]
    fn on_click_fires() {
        let node: Node<TestApp> = Node::new()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| { app.clicked = true; });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        let (_, result) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert!(app.clicked);
        assert!(result.click_consumed_by_ui());
    }

    #[test]
    fn on_click_outside_does_not_fire() {
        let node: Node<TestApp> = Node::new()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| { app.clicked = true; });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(50.0, 50.0);
        let (_, result) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert!(!app.clicked);
        assert!(!result.click_consumed_by_ui());
    }

    #[test]
    fn on_hover_fires_on_parent_and_child() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_hover(|app: &mut TestApp, _state| { app.count += 1; })
            .children(vec![
                Node::new()
                    .size(10, 10)
                    .color(RED)
                    .on_hover(|app: &mut TestApp, _state| { app.count += 10; }),
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(5.0, 5.0); // inside child (and parent)
        evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(app.count, 11); // both fire
    }

    #[test]
    fn action_deepest_wins() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_click(|app: &mut TestApp, _state| { app.count += 1; })
            .children(vec![
                Node::new()
                    .size(10, 10)
                    .color(RED)
                    .on_click(|app: &mut TestApp, _state| { app.count += 10; }),
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0); // inside child
        evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(app.count, 10); // only child fires
    }

    #[test]
    fn action_falls_through_to_parent_if_child_has_no_handler() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_click(|app: &mut TestApp, _state| { app.count += 1; })
            .children(vec![
                Node::new().size(10, 10).color(RED), // no on_click
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(app.count, 1); // parent handles it
    }

    #[test]
    fn style_inheritance() {
        let node: Node<TestApp> = col()
            .color(RED)
            .children(vec![
                Node::new().size(5, 5), // inherits RED
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        // Child should draw with inherited RED
        assert!(!cells.is_empty());
        assert_eq!(cells[0].color, RED.to_array());
    }

    #[test]
    fn style_override() {
        let node: Node<TestApp> = col()
            .color(RED)
            .children(vec![
                Node::new().size(5, 5).color(BLUE), // overrides
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn style_object_batch_apply() {
        let shared = Style::new().color(GREEN).gap(4);
        let node: Node<TestApp> = row().style(shared).children(vec![
            Node::new().size(5, 5),
            Node::new().size(5, 5),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let resolved = layout_node(&node, &ctx, &Style::default(), &mut idx);
        // gap=4 applied: 5 + 4 + 5 = 14
        assert_eq!(resolved.rect.width(), 14);
    }

    #[test]
    fn z_depth_parent_behind_children() {
        let node: Node<TestApp> = col()
            .size(10, 10)
            .color(GREY)
            .children(vec![
                Node::new().size(5, 5).color(RED),
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        // Find parent cell (GREY) and child cell (RED)
        let child_cell = cells.iter().find(|c| c.color == RED.to_array()).unwrap();
        let parent_cell = cells.iter().find(|c| c.color == GREY.to_array()).unwrap();
        assert!(child_cell.position.z > parent_cell.position.z);
    }

    #[test]
    fn consumed_flag_per_button() {
        let node: Node<TestApp> = Node::new()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| { app.clicked = true; });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        let (_, result) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert!(result.consumed_by_ui(MouseButton::Left));
        assert!(!result.consumed_by_ui(MouseButton::Right));
        assert!(result.any_consumed_by_ui());
    }

    #[test]
    fn positioned_node_tested_before_flow() {
        let node: Node<TestApp> = col()
            .size(50, 50)
            .children(vec![
                Node::new()
                    .size(20, 20)
                    .color(RED)
                    .on_click(|app: &mut TestApp, _state| { app.count += 1; }),
                Node::new()
                    .size(20, 20)
                    .color(BLUE)
                    .absolute(0, 0)
                    .on_click(|app: &mut TestApp, _state| { app.count += 10; }),
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0); // inside both
        evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(app.count, 10); // positioned wins
    }

    #[test]
    fn scroll_handler() {
        let node: Node<TestApp> = Node::new()
            .size(20, 20)
            .color(RED)
            .on_scroll(|app: &mut TestApp, _state, delta| { app.scroll_amount = delta; });
        let mut app = TestApp::new();
        let mut state = make_state();
        let mut input = input_at(5.0, 5.0);
        input.scroll_delta = 3.0;
        let (_, result) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert_eq!(app.scroll_amount, 3.0);
        assert!(result.scroll_consumed_by_ui());
    }

    #[test]
    fn button_convenience() {
        let node: Node<TestApp> = button()
            .size(8, 4)
            .color(RED)
            .hover_color(BLUE)
            .on_click(|app: &mut TestApp, _state| { app.clicked = true; });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(4.0, 2.0);
        let (cells, result) = evaluate(node, &mut app, &mut state, &input, 256, 256);
        assert!(app.clicked);
        assert!(result.click_consumed_by_ui());
        // Hovered at click point, should use hover_color
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn row_col_convenience() {
        let r: Node<TestApp> = row().children(vec![
            Node::new().size(5, 10),
            Node::new().size(5, 10),
        ]);
        let c: Node<TestApp> = col().children(vec![
            Node::new().size(10, 5),
            Node::new().size(10, 5),
        ]);
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: 256,
            available_h: 256,
        };
        let mut idx = 0;
        let r_resolved = layout_node(&r, &ctx, &Style::default(), &mut idx);
        idx = 0;
        let c_resolved = layout_node(&c, &ctx, &Style::default(), &mut idx);
        assert_eq!(r_resolved.rect.width(), 10);
        assert_eq!(r_resolved.rect.height(), 10);
        assert_eq!(c_resolved.rect.width(), 10);
        assert_eq!(c_resolved.rect.height(), 10);
    }
}
