use crate::cell::Cell;
use crate::color::Color;
use crate::element::DragInfo;
use crate::font::Font;
use crate::input::MouseButton;
use crate::rect::Rect;
use crate::runner::{App, State};
use crate::{Drawable, Shape, Text};
use tiles_macros::Builders;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Axis {
    #[default]
    Column,
    Row,
}

/// How a single axis resolves its size
#[derive(Clone, Copy, Debug, Default)]
pub enum Sizing {
    /// Exactly this many pixels
    Fixed(u32),
    /// Shrink-wrap children (or 0 if no children)
    #[default]
    Shrink,
    /// Take up remaining parent space, divided equally among siblings also Fill
    Fill,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Position {
    #[default]
    Flow,
    Relative(i32, i32),
    Absolute(i32, i32),
}

// --- Styles ---

// 1. auto mark field as required builder unless marked as custom, or marked as omit
// 2. mark field as inheritable, and generate a function which allows a Style to inherit from another style
//      if is None

#[derive(Clone, Debug, Default, Builders)]
#[builders(forward(to = "Node<A: App>", via = "style"))]
pub struct Style {
    #[builder(dual_variant(name = "size", variant = "Fixed", args = "w: u32, h: u32",))]
    #[builder(variant(name = "fill_w", variant = "Fill"))]
    #[builder(variant(name = "shrink_w", variant = "Shrink"))]
    #[builder(variant(name = "width", variant = "Fixed", args = "width: u32"))]
    pub w: Sizing,
    #[builder(variant(name = "fill_h", variant = "Fill"))]
    #[builder(variant(name = "shrink_h", variant = "Shrink"))]
    #[builder(variant(name = "height", variant = "Fixed", args = "height: u32"))]
    pub h: Sizing,
    #[builder]
    pub axis: Axis,
    #[builder]
    pub gap: Option<u32>,
    #[builder]
    pub padding: Option<u32>,
    #[builder(variant(name = "relative", variant = "Relative", args = "x: i32, y: i32"))]
    #[builder(variant(name = "absolute", variant = "Absolute", args = "x: i32, y: i32"))]
    pub position: Position,
    #[builder]
    pub z_index: i32,
    #[builder]
    pub color: Option<Color>,
    #[builder]
    pub hover_color: Option<Color>,
    #[builder]
    pub pressed_color: Option<Color>,
    #[builder]
    pub text_color: Option<Color>,
    #[builder]
    pub hover_text_color: Option<Color>,
    #[builder]
    pub pressed_text_color: Option<Color>,
    #[builder]
    pub font: Option<&'static Font>,
}

// --- Handlers ---

#[derive(Builders)]
#[builders(forward(to = "Node<A: App>", via = "handlers"))]
pub struct Handlers<A: App> {
    #[builder]
    pub on_hover: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_enter: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_leave: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_double_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_press: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_release: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_right_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_hold: Option<Box<dyn Fn(&mut A, &mut State)>>,
    #[builder]
    pub on_drag: Option<Box<dyn Fn(&mut A, &mut State, DragInfo)>>,
    #[builder]
    pub on_drag_end: Option<Box<dyn Fn(&mut A, &mut State, DragInfo)>>,
    #[builder]
    pub on_scroll: Option<Box<dyn Fn(&mut A, &mut State, f32)>>,
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

// --- Node types ---
pub enum NodeContent<A: App> {
    Children(Vec<Node<A>>),
    Text(String),
}

pub struct Node<A: App> {
    id: String,
    style: Style,
    handlers: Handlers<A>,
    content: NodeContent<A>,
}

impl<A: App> From<Vec<Node<A>>> for NodeContent<A> {
    fn from(children: Vec<Node<A>>) -> Self {
        NodeContent::Children(children)
    }
}

impl<A: App> From<String> for NodeContent<A> {
    fn from(string: String) -> Self {
        NodeContent::Text(string)
    }
}

/// Size after layout
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Size after layout
pub struct SizedNode<A: App> {
    pub node: Node<A>,
    pub size: Size,
    pub children: Vec<SizedNode<A>>,
}

impl<A: App> Node<A> {
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn children<I: Into<Node<A>>>(mut self, children: Vec<I>) -> Self {
        match self.content {
            NodeContent::Children(_) => {
                self.content =
                    NodeContent::Children(children.into_iter().map(Into::into).collect());
                self
            }
            NodeContent::Text(_) => self,
        }
    }

    // fn size_hint(&self, parent_font: &'static Font) -> (u32, u32) {
    //     if self.style.position != Position::Flow {
    //         return (0, 0);
    //     }

    //     let font = self.style.font.unwrap_or(parent_font);
    //     // let w = self.style.w.unwrap_or(0);
    //     // let h = self.style.h.unwrap_or(0);
    //     match &self.content {
    //         NodeContent::Children(children) => {
    //             let mut x = 0;
    //             let mut y = 0;
    //             for child in children {
    //                 let (c_w, c_h) = child.size_hint(font);
    //                 match &self.style.axis.unwrap_or_default() {
    //                     Axis::Column => {
    //                         x = x.max(c_w);
    //                         y = y + self.style.gap.unwrap_or(0) + c_h;
    //                     }
    //                     Axis::Row => {
    //                         y = y.max(c_h);
    //                         x = x + self.style.gap.unwrap_or(0) + c_w;
    //                     }
    //                 }
    //             }
    //             x = x + 2 * self.style.padding.unwrap_or(0);
    //             y = y + 2 * self.style.padding.unwrap_or(0);
    //             return (x.max(w), y.max(h));
    //         }
    //         NodeContent::Text(text) => {
    //             let text = Text::new(&self.style.font.unwrap_or(parent_font), text)
    //                 .anchor(crate::AnchorBox::Tight, crate::AnchorCorner::TopLeft);
    //             let padding = self.style.padding.unwrap_or(0) as i32;
    //             let text_bounding_rect = text.bounds().expand(padding);
    //             return (
    //                 text_bounding_rect.width().max(w),
    //                 text_bounding_rect.height().max(h),
    //             );
    //         }
    //     }
    // }

    pub(crate) fn layout(self, screen_w: u32, screen_h: u32) -> ResolvedNode<A> {
        let ctx = LayoutCtx {
            origin_x: 0,
            origin_y: 0,
            available_w: screen_w,
            available_h: screen_h,
        };
        self.layout_with_context(&ctx, None.unwrap_or_default())
    }

    fn layout_with_context(self, ctx: &LayoutCtx, parent_font: &'static Font) -> ResolvedNode<A> {
        // inherited
        let font = self.style.font.unwrap_or(parent_font);

        match self.content {
            NodeContent::Children(children) => {
                let padding = self.style.padding.unwrap_or(0);
                let axis = self.style.axis;
                let gap = self.style.gap.unwrap_or(0);

                let own_w = match self.style.w {
                    Sizing::Fixed(w) => Some(w),
                    Sizing::Shrink => None,
                    Sizing::Fill => Some(ctx.available_w),
                };

                let own_h = match self.style.h {
                    Sizing::Fixed(h) => Some(h),
                    Sizing::Shrink => None,
                    Sizing::Fill => Some(ctx.available_h),
                };

                let (origin_x, origin_y) = match self.style.position {
                    Position::Flow => (ctx.origin_x, ctx.origin_y),
                    Position::Relative(rx, ry) => (ctx.origin_x + rx, ctx.origin_y + ry),
                    Position::Absolute(ax, ay) => (ax, ay),
                };

                let content_available_w =
                    own_w.unwrap_or(ctx.available_w).saturating_sub(padding * 2);
                let content_available_h =
                    own_h.unwrap_or(ctx.available_h).saturating_sub(padding * 2);
                let content_origin_x = origin_x + padding as i32;
                let content_origin_y = origin_y + padding as i32;

                let mut resolved_children = Vec::new();
                let mut absolute_children = Vec::new();
                let mut relative_children = Vec::new();
                let mut cursor_x: u32 = 0;
                let mut cursor_y: u32 = 0;
                let mut max_cross: u32 = 0;
                let mut child_count: u32 = 0;

                for child in children {
                    let child_ctx = LayoutCtx {
                        origin_x: content_origin_x + cursor_x as i32,
                        origin_y: content_origin_y + cursor_y as i32,
                        available_w: content_available_w,
                        available_h: content_available_h,
                    };
                    let resolved_child = child.layout_with_context(&child_ctx, font);

                    match resolved_child.base_style.position {
                        Position::Relative(_, _) => {
                            relative_children.push(resolved_child);
                            continue;
                        }
                        Position::Absolute(_, _) => {
                            absolute_children.push(resolved_child);
                            continue;
                        }
                        _ => {}
                    }

                    let child_w = resolved_child.rect.width();
                    let child_h = resolved_child.rect.height();
                    match axis {
                        Axis::Row => {
                            cursor_x = cursor_x.saturating_add(child_w).saturating_add(gap);
                            max_cross = max_cross.max(child_h);
                        }
                        Axis::Column => {
                            cursor_y = cursor_y.saturating_add(child_h).saturating_add(gap);
                            max_cross = max_cross.max(child_w);
                        }
                    }
                    child_count += 1;
                    resolved_children.push(resolved_child);
                }

                let (content_w, content_h) = match axis {
                    Axis::Row => {
                        let main = if child_count > 0 {
                            cursor_x.saturating_sub(gap)
                        } else {
                            0
                        };
                        (main, max_cross)
                    }
                    Axis::Column => {
                        let main = if child_count > 0 {
                            cursor_y.saturating_sub(gap)
                        } else {
                            0
                        };
                        (max_cross, main)
                    }
                };

                let final_w = own_w.unwrap_or(content_w.saturating_add(padding * 2));
                let final_h = own_h.unwrap_or(content_h.saturating_add(padding * 2));
                let rect = Rect::from_top_left(origin_x as f32, origin_y as f32, final_w, final_h);

                resolved_children.append(&mut relative_children);
                resolved_children.append(&mut absolute_children);

                ResolvedNode {
                    #[cfg(test)]
                    id: self.id,
                    rect,
                    base_style: self.style,
                    text: None,
                    children: resolved_children,
                    handlers: self.handlers,
                }
            }
            NodeContent::Text(text) => {
                let padding = self.style.padding.unwrap_or(0) as i32;

                let (origin_x, origin_y) = match self.style.position {
                    Position::Flow => (ctx.origin_x, ctx.origin_y),
                    Position::Relative(rx, ry) => (ctx.origin_x + rx, ctx.origin_y + ry),
                    Position::Absolute(ax, ay) => (ax, ay),
                };

                let text = Text::new(font, text)
                    .anchor(crate::AnchorBox::Highlight, crate::AnchorCorner::TopLeft)
                    .position((origin_x + padding) as f32, (origin_y + padding) as f32);
                ResolvedNode {
                    #[cfg(test)]
                    id: self.id,
                    rect: text.rect().expand(padding),
                    base_style: self.style,
                    text: Some(text),
                    children: Vec::new(),
                    handlers: self.handlers,
                }
            }
        }
    }
}

// --- Convenience constructors ---

pub fn pane<A: App>() -> Node<A> {
    Node {
        id: String::default(),
        style: Style::default(),
        content: Vec::new().into(),
        handlers: Handlers::default(),
    }
}

pub fn row<A: App>() -> Node<A> {
    pane().axis(Axis::Row)
}

pub fn col<A: App>() -> Node<A> {
    pane().axis(Axis::Column)
}

pub fn text<A: App>(content: impl Into<String>) -> Node<A> {
    Node {
        id: String::default(),
        style: Style::default(),
        content: content.into().into(),
        handlers: Handlers::default(),
    }
}

// --- Layout ---

struct LayoutCtx {
    origin_x: i32,
    origin_y: i32,
    available_w: u32,
    available_h: u32,
}

pub(crate) struct ResolvedNode<A: App> {
    #[cfg(test)]
    id: String,
    rect: Rect,
    base_style: Style,
    text: Option<Text>,
    children: Vec<ResolvedNode<A>>,
    handlers: Handlers<A>,
}

// --- Evaluate ---

impl<A: App> ResolvedNode<A> {
    pub(crate) fn evaluate(self, app: &mut A, state: &mut State) -> (Vec<Cell>, EvaluateResult) {
        let mut cells = Vec::new();
        let mut consumed = ConsumedState::new();
        self.evaluate_recursive(
            app,
            state,
            &mut cells,
            &mut consumed,
            Some(Color::hex(0xFFFFFF)),
            0.0,
        );
        cells.reverse();
        (cells, EvaluateResult { consumed })
    }

    fn evaluate_recursive(
        self,
        app: &mut A,
        state: &mut State,
        cells: &mut Vec<Cell>,
        consumed: &mut ConsumedState,
        text_color: Option<Color>,
        depth: f32,
    ) {
        let hit = state.test_shape_screen(&self.rect);

        let color = if hit.is_down() {
            self.base_style
                .pressed_color
                .or(self.base_style.hover_color)
                .or(self.base_style.color)
        } else if hit.is_hovered() {
            self.base_style.hover_color.or(self.base_style.color)
        } else {
            self.base_style.color
        };

        let text_color = if hit.is_down() {
            self.base_style
                .pressed_text_color
                .or(self.base_style.hover_text_color)
                .or(self.base_style.text_color)
        } else if hit.is_hovered() {
            self.base_style
                .hover_text_color
                .or(self.base_style.text_color)
        } else {
            self.base_style.text_color
        }
        .or(text_color);

        for node in self.children.into_iter().rev() {
            node.evaluate_recursive(app, state, cells, consumed, text_color, depth + 1.0);
        }

        if hit.is_hovered() {
            if let Some(f) = self.handlers.on_hover {
                f(app, state);
            }
        }
        if hit.has_entered() {
            if let Some(f) = self.handlers.on_enter {
                f(app, state);
            }
        }
        if hit.has_left() {
            if let Some(f) = self.handlers.on_leave {
                f(app, state);
            }
        }

        if !consumed.left {
            if hit.is_clicked() {
                if let Some(f) = self.handlers.on_click {
                    f(app, state);
                    consumed.left = true;
                }
            }
            if hit.is_double_clicked() {
                if let Some(f) = self.handlers.on_double_click {
                    f(app, state);
                    consumed.left = true;
                }
            }
            if hit.is_pressed() {
                if let Some(f) = self.handlers.on_press {
                    f(app, state);
                    consumed.left = true;
                }
            }
            if hit.is_released() {
                if let Some(f) = self.handlers.on_release {
                    f(app, state);
                    consumed.left = true;
                }
            }
            if hit.is_held().is_some() {
                if let Some(f) = self.handlers.on_hold {
                    f(app, state);
                    consumed.left = true;
                }
            }
            if let Some(drag) = hit.is_dragging() {
                if let Some(f) = self.handlers.on_drag {
                    f(app, state, drag);
                    consumed.left = true;
                }
            }
            if let Some(drag) = hit.is_drag_end() {
                if let Some(f) = self.handlers.on_drag_end {
                    f(app, state, drag);
                    consumed.left = true;
                }
            }
        }
        if !consumed.right {
            if hit.is_right_clicked() {
                if let Some(f) = self.handlers.on_right_click {
                    f(app, state);
                    consumed.right = true;
                }
            }
        }
        if !consumed.scroll {
            if let Some(delta) = hit.is_scrolling() {
                if let Some(f) = self.handlers.on_scroll {
                    f(app, state, delta);
                    consumed.scroll = true;
                }
            }
        }

        // Draw pane background
        if let Some(color) = color {
            self.rect.fill().color(color).emit_cells(&mut |mut c| {
                c.position.z = depth;
                cells.push(c);
            });
        }

        // Draw text glyphs
        if let (Some(text), Some(text_color)) = (self.text, text_color) {
            text.color(text_color).emit_cells(&mut |mut c| {
                c.position.z = depth + 0.5;
                cells.push(c);
            });
        }
    }

    #[cfg(test)]
    pub(crate) fn find_child_by_id(&self, id: &str) -> Option<&Self> {
        self.children.iter().find_map(|c| (c.id == id).then_some(c))
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

// --- Public API ---

pub struct EvaluateResult {
    consumed: ConsumedState,
}

impl EvaluateResult {
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
        count: i32,
        scroll_amount: f32,
    }

    impl TestApp {
        fn new() -> Self {
            Self {
                clicked: false,
                count: 0,
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

    fn eval(
        node: Node<TestApp>,
        app: &mut TestApp,
        state: &mut State,
    ) -> (Vec<Cell>, EvaluateResult) {
        node.layout(256, 256).evaluate(app, state)
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
        let node: Node<TestApp> = row();
        let resolved: ResolvedNode<_> = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 0);
        assert_eq!(resolved.rect.height(), 0);
    }

    #[test]
    fn explicit_size() {
        let node: Node<TestApp> = row().size(10, 5);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 10);
        assert_eq!(resolved.rect.height(), 5);
    }

    #[test]
    fn fill_w_takes_available() {
        let node: Node<TestApp> = row().fill_w().height(10);
        let resolved = node.layout(100, 256);
        assert_eq!(resolved.rect.width(), 100);
        assert_eq!(resolved.rect.height(), 10);
    }

    #[test]
    fn column_layout_stacks_vertically() {
        let node: Node<TestApp> = col().children(vec![
            row().size(10, 5),
            row().size(10, 5),
            row().size(10, 5),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 10);
        assert_eq!(resolved.rect.height(), 15);
        assert_eq!(resolved.children[0].rect.y(), 0.0);
        assert_eq!(resolved.children[1].rect.y(), 5.0);
        assert_eq!(resolved.children[2].rect.y(), 10.0);
    }

    #[test]
    fn row_layout_stacks_horizontally() {
        let node: Node<TestApp> = row().children(vec![row().size(10, 5), row().size(10, 5)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 20);
        assert_eq!(resolved.rect.height(), 5);
        assert_eq!(resolved.children[0].rect.x(), 0.0);
        assert_eq!(resolved.children[1].rect.x(), 10.0);
    }

    #[test]
    fn gap_between_children() {
        let node: Node<TestApp> = row()
            .gap(4)
            .children(vec![row().size(10, 5), row().size(10, 5)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 24);
        assert_eq!(resolved.children[1].rect.x(), 14.0);
    }

    #[test]
    fn padding_offsets_children() {
        let node: Node<TestApp> = col().padding(3).children(vec![row().size(4, 4)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 10); // 4 + 3*2
        assert_eq!(resolved.rect.height(), 10);
        assert_eq!(resolved.children[0].rect.x(), 3.0);
        assert_eq!(resolved.children[0].rect.y(), 3.0);
    }

    #[test]
    fn nested_layout() {
        let node: Node<TestApp> = col().padding(2).children(vec![row()
            .gap(2)
            .children(vec![row().size(5, 5), row().size(5, 5)])]);
        let resolved = node.layout(256, 256);
        // Inner row: 5+2+5=12 wide, 5 tall
        // Outer: 12+4=16 wide, 5+4=9 tall
        assert_eq!(resolved.rect.width(), 16);
        assert_eq!(resolved.rect.height(), 9);
    }

    #[test]
    fn absolute_position_skips_cursor() {
        let node: Node<TestApp> = col().children(vec![
            row().size(10, 10),
            row().id("abs").size(5, 5).absolute(50, 50),
            row().id("flow2").size(10, 10),
        ]);
        let resolved = node.layout(256, 256);
        // Absolute node doesn't affect parent size
        assert_eq!(resolved.rect.height(), 20); // only 2 flow children
                                                // Absolute node at (50, 50)
        let abs_child = resolved.find_child_by_id("abs").unwrap();
        assert_eq!(abs_child.rect.x(), 50.0);
        assert_eq!(abs_child.rect.y(), 50.0);
        // Third child at y=10 (not y=15)
        assert_eq!(resolved.find_child_by_id("flow2").unwrap().rect.y(), 10.0);
    }

    #[test]
    fn relative_position() {
        let node: Node<TestApp> = col()
            .padding(5)
            .children(vec![row().size(10, 10).relative(3, 3)]);
        let resolved = node.layout(256, 256);
        // Child positioned at parent cursor (5,5) + relative offset (3,3) = (8,8)
        let child = &resolved.children[0];
        assert_eq!(child.rect.x(), 8.0);
        assert_eq!(child.rect.y(), 8.0);
    }

    // --- Draw tests ---

    #[test]
    fn colored_node_emits_cells() {
        let node: Node<TestApp> = row().size(3, 2).color(RED);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(cells.len(), 6);
        assert_eq!(cells[0].color, RED.to_array());
    }

    #[test]
    fn uncolored_node_emits_no_cells() {
        let node: Node<TestApp> = row().size(3, 2);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(cells.len(), 0);
    }

    #[test]
    fn hover_color_on_hover() {
        let node: Node<TestApp> = row().size(10, 10).color(RED).hover_color(BLUE);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(5.0, 5.0); // inside
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn normal_color_when_not_hovered() {
        let node: Node<TestApp> = row().size(10, 10).color(RED).hover_color(BLUE);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(50.0, 50.0); // outside
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(cells[0].color, RED.to_array());
    }

    // --- Handler tests ---

    #[test]
    fn on_click_fires() {
        let node = row()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| {
                app.clicked = true;
            });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        let (_, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert!(app.clicked);
        assert!(result.click_consumed_by_ui());
    }

    #[test]
    fn on_click_outside_does_not_fire() {
        let node = row()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| {
                app.clicked = true;
            });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(50.0, 50.0);
        let (_, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert!(!app.clicked);
        assert!(!result.click_consumed_by_ui());
    }

    #[test]
    fn on_hover_fires_on_parent_and_child() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_hover(|app: &mut TestApp, _state| {
                app.count += 1;
            })
            .children(vec![row().size(10, 10).color(RED).on_hover(
                |app: &mut TestApp, _state| {
                    app.count += 10;
                },
            )]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(5.0, 5.0); // inside child (and parent)
        {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(app.count, 11); // both fire
    }

    #[test]
    fn action_deepest_wins() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_click(|app: &mut TestApp, _state| {
                app.count += 1;
            })
            .children(vec![row().size(10, 10).color(RED).on_click(
                |app: &mut TestApp, _state| {
                    app.count += 10;
                },
            )]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0); // inside child
        {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(app.count, 10); // only child fires
    }

    #[test]
    fn action_falls_through_to_parent_if_child_has_no_handler() {
        let node: Node<TestApp> = col()
            .size(20, 20)
            .color(GREY)
            .on_click(|app: &mut TestApp, _state| {
                app.count += 1;
            })
            .children(vec![
                row().size(10, 10).color(RED), // no on_click
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(app.count, 1); // parent handles it
    }

    #[test]
    fn style_inheritance() {
        let node: Node<TestApp> = col().color(RED).children(vec![
            row().size(5, 5), // inherits RED
        ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        // Child should draw with inherited RED
        assert!(!cells.is_empty());
        assert_eq!(cells[0].color, RED.to_array());
    }

    #[test]
    fn style_override() {
        let node: Node<TestApp> = col().color(RED).children(vec![
            row().size(5, 5).color(BLUE), // overrides
        ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(cells.last().unwrap().color, BLUE.to_array());
    }

    #[test]
    fn gap_applied_directly() {
        let node: Node<TestApp> = row()
            .color(GREEN)
            .gap(4)
            .children(vec![row().size(5, 5), row().size(5, 5)]);
        let resolved = node.layout(256, 256);
        // gap=4 applied: 5 + 4 + 5 = 14
        assert_eq!(resolved.rect.width(), 14);
    }

    #[test]
    fn z_depth_parent_behind_children() {
        let node: Node<TestApp> = col()
            .size(10, 10)
            .color(GREY)
            .children(vec![row().size(5, 5).color(RED)]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_at(100.0, 100.0);
        let (cells, _) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        // Find parent cell (GREY) and child cell (RED)
        let child_cell = cells.iter().find(|c| c.color == RED.to_array()).unwrap();
        let parent_cell = cells.iter().find(|c| c.color == GREY.to_array()).unwrap();
        assert!(child_cell.position.z > parent_cell.position.z);
    }

    #[test]
    fn consumed_flag_per_button() {
        let node = row()
            .size(10, 10)
            .color(RED)
            .on_click(|app: &mut TestApp, _state| {
                app.clicked = true;
            });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0);
        let (_, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert!(result.consumed_by_ui(MouseButton::Left));
        assert!(!result.consumed_by_ui(MouseButton::Right));
        assert!(result.any_consumed_by_ui());
    }

    #[test]
    fn positioned_node_tested_before_flow() {
        let node =
            col().size(50, 50).children(vec![
                row()
                    .size(20, 20)
                    .color(RED)
                    .on_click(|app: &mut TestApp, _state| {
                        app.count += 1;
                    }),
                row().size(20, 20).color(BLUE).absolute(0, 0).on_click(
                    |app: &mut TestApp, _state| {
                        app.count += 10;
                    },
                ),
            ]);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(5.0, 5.0); // inside both
        {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(app.count, 10); // positioned wins
    }

    #[test]
    fn scroll_handler() {
        let node = row()
            .size(20, 20)
            .color(RED)
            .on_scroll(|app: &mut TestApp, _state, delta| {
                app.scroll_amount = delta;
            });
        let mut app = TestApp::new();
        let mut state = make_state();
        let mut input = input_at(5.0, 5.0);
        input.scroll_delta = 3.0;
        let (_, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert_eq!(app.scroll_amount, 3.0);
        assert!(result.scroll_consumed_by_ui());
    }

    #[test]
    fn button_convenience() {
        let node =
            pane()
                .size(8, 4)
                .color(RED)
                .hover_color(BLUE)
                .on_click(|app: &mut TestApp, _state| {
                    app.clicked = true;
                });
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(4.0, 2.0);
        let (cells, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert!(app.clicked);
        assert!(result.click_consumed_by_ui());
        // Hovered at click point, should use hover_color
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn row_col_convenience() {
        let r: Node<TestApp> = row().children(vec![row().size(5, 10), row().size(5, 10)]);
        let c: Node<TestApp> = col().children(vec![row().size(10, 5), row().size(10, 5)]);
        let r_resolved = r.layout(256, 256);
        let c_resolved = c.layout(256, 256);
        assert_eq!(r_resolved.rect.width(), 10);
        assert_eq!(r_resolved.rect.height(), 10);
        assert_eq!(c_resolved.rect.width(), 10);
        assert_eq!(c_resolved.rect.height(), 10);
    }

    // --- Macro tests ---

    #[test]
    fn macro_simple_nodes() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            row().size(5, 5).color(RED);
            row().size(3, 3).color(BLUE);
        };
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn macro_nested_children() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            row().gap(4) {
                row().size(5, 5).color(RED);
                row().size(5, 5).color(BLUE);
            }
        };
        assert_eq!(children.len(), 1);
        let resolved = children.into_iter().next().unwrap().layout(256, 256);
        // 5 + 4 + 5 = 14
        assert_eq!(resolved.rect.width(), 14);
    }

    #[test]
    fn macro_if_control_flow() {
        use crate::ui;
        let show = true;
        let children: Vec<Node<TestApp>> = ui! {
            row().size(5, 5).color(RED);
            @ if show {
                row().size(3, 3).color(BLUE);
            }
        };
        assert_eq!(children.len(), 2);

        let show = false;
        let children: Vec<Node<TestApp>> = ui! {
            row().size(5, 5).color(RED);
            @ if show {
                row().size(3, 3).color(BLUE);
            }
        };
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn macro_for_loop() {
        use crate::ui;
        let colors = [RED, BLUE, GREEN];
        let children: Vec<Node<TestApp>> = ui! {
            @ for c in colors {
                row().size(5, 5).color(c);
            }
        };
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn macro_raw_escape() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            row().size(5, 5).color(RED);
            |c| {
                c.push(row().size(3, 3).color(BLUE).into());
                c.push(row().size(3, 3).color(GREEN).into());
            }
        };
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn macro_with_handlers() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            pane()
                .size(5, 3)
                .color(RED)
                .hover_color(BLUE)
                .on_click(|app: &mut TestApp, _state| { app.clicked = true; });
        };
        assert_eq!(children.len(), 1);
        let mut app = TestApp::new();
        let mut state = make_state();
        let input = input_with_click_at(2.0, 1.0);
        let node = col::<TestApp>().children(children);
        let (_, result) = {
            state.set_input(input);
            eval(node, &mut app, &mut state)
        };
        assert!(app.clicked);
        assert!(result.click_consumed_by_ui());
    }

    #[test]
    fn macro_deeply_nested() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            col().padding(2) {
                row().gap(2) {
                    row().size(5, 5).color(RED);
                    row().size(5, 5).color(BLUE);
                }
                row().size(10, 3).color(GREEN);
            }
        };
        assert_eq!(children.len(), 1);
        let resolved = children.into_iter().next().unwrap().layout(256, 256);
        // row: 5+2+5=12, col content: max(12,10)=12 wide, 5+3=8 tall
        // with padding 2: 12+4=16 wide, 8+4=12 tall
        assert_eq!(resolved.rect.width(), 16);
        assert_eq!(resolved.rect.height(), 12);
    }

    #[test]
    fn macro_for_with_expr_iter() {
        use crate::ui;
        let items = vec![(5, RED), (3, BLUE), (7, GREEN)];
        let children: Vec<Node<TestApp>> = ui! {
            @ for (size, color) in items.iter().copied() {
                row().size(size, size).color(color)
            }
        };
        assert_eq!(children.len(), 3);
    }

    // --- Text tests ---

    #[test]
    fn text_node_intrinsic_size() {
        let node: Node<TestApp> = col().children(vec![text("Hi")]);
        let resolved = node.layout(256, 256);
        let child = &resolved.children[0];
        assert!(child.rect.width() > 0);
        assert!(child.rect.height() > 0);
    }

    #[test]
    fn text_node_with_padding() {
        let node: Node<TestApp> = col().children(vec![text("A").padding(3)]);
        let resolved = node.layout(256, 256);
        let child = &resolved.children[0];
        let no_pad: Node<TestApp> = col().children(vec![text("A")]);
        let no_pad_resolved = no_pad.layout(256, 256);
        let no_pad_child = &no_pad_resolved.children[0];
        assert_eq!(child.rect.width(), no_pad_child.rect.width() + 6);
        assert_eq!(child.rect.height(), no_pad_child.rect.height() + 6);
    }

    #[test]
    fn text_node_omits_uncolored_cells() {
        let node: Node<TestApp> = col().children(vec![text("I")]);
        let mut app = TestApp::new();
        let mut state = make_state();
        state.set_input(input_at(100.0, 100.0));
        let (cells, _) = node.layout(256, 256).evaluate(&mut app, &mut state);
        assert!(cells.is_empty());
    }

    #[test]
    fn text_inherits_text_color() {
        let node: Node<TestApp> = col().text_color(RED).children(vec![text("I")]);
        let mut app = TestApp::new();
        let mut state = make_state();
        state.set_input(input_at(100.0, 100.0));
        let (cells, _) = node.layout(256, 256).evaluate(&mut app, &mut state);
        assert!(!cells.is_empty());
        assert_eq!(cells[0].color, RED.to_array());
    }

    #[test]
    fn text_own_color_overrides_inherited() {
        let node: Node<TestApp> = col()
            .text_color(RED)
            .children(vec![text("I").text_color(BLUE)]);
        let mut app = TestApp::new();
        let mut state = make_state();
        state.set_input(input_at(100.0, 100.0));
        let (cells, _) = node.layout(256, 256).evaluate(&mut app, &mut state);
        assert!(!cells.is_empty());
        assert_eq!(cells[0].color, BLUE.to_array());
    }

    #[test]
    fn text_inherits_font() {
        use crate::font::TOM_THUMB_3X5;
        let node: Node<TestApp> = col().font(&TOM_THUMB_3X5).children(vec![text("A")]);
        let resolved = node.layout(256, 256);
        let child = &resolved.children[0];
        assert!(std::ptr::eq(child.base_style.font.unwrap(), &TOM_THUMB_3X5));
    }

    #[test]
    fn text_in_macro() {
        use crate::ui;
        let children: Vec<Node<TestApp>> = ui! {
            col().text_color(RED) {
                text("hello");
            }
        };
        assert_eq!(children.len(), 1);
    }
}
