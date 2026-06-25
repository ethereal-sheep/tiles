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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
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
    pub axis: Axis,
    pub gap: Option<u32>,
    pub padding: Option<u32>,
    #[builder(variant(name = "justify_start", variant = "Start"))]
    #[builder(variant(name = "justify_center", variant = "Center"))]
    #[builder(variant(name = "justify_end", variant = "End"))]
    #[builder(variant(name = "justify_full", variant = "SpaceBetween"))]
    pub justify: Justify,
    #[builder(variant(name = "align_start", variant = "Start"))]
    #[builder(variant(name = "align_center", variant = "Center"))]
    #[builder(variant(name = "align_end", variant = "End"))]
    pub align: Align,
    #[builder(variant(name = "relative", variant = "Relative", args = "x: i32, y: i32"))]
    #[builder(variant(name = "absolute", variant = "Absolute", args = "x: i32, y: i32"))]
    pub position: Position,
    pub z_index: i32,
    pub color: Option<Color>,
    pub hover_color: Option<Color>,
    pub pressed_color: Option<Color>,
    pub text_color: Option<Color>,
    pub hover_text_color: Option<Color>,
    pub pressed_text_color: Option<Color>,
    pub font: Option<&'static Font>,
}

// --- Handlers ---

#[derive(Builders)]
#[builders(forward(to = "Node<A: App>", via = "handlers"))]
pub struct Handlers<A: App> {
    pub on_hover: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_enter: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_leave: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_double_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_press: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_release: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_right_click: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_hold: Option<Box<dyn Fn(&mut A, &mut State)>>,
    pub on_drag: Option<Box<dyn Fn(&mut A, &mut State, DragInfo)>>,
    pub on_drag_end: Option<Box<dyn Fn(&mut A, &mut State, DragInfo)>>,
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
pub enum NodeContent<N, T> {
    Children(Vec<N>),
    Text(T),
}

pub struct Node<A: App> {
    id: String,
    style: Style,
    handlers: Handlers<A>,
    content: NodeContent<Self, String>,
}

impl<N, T> From<Vec<N>> for NodeContent<N, T> {
    fn from(children: Vec<N>) -> Self {
        NodeContent::Children(children)
    }
}

impl<N> From<String> for NodeContent<N, String> {
    fn from(string: String) -> Self {
        NodeContent::Text(string)
    }
}

/// Resolved dimensions after the size pass
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Intermediate tree produced by the pre-process pass, consumed by the size pass
struct ProcessedNode<A: App> {
    #[cfg(test)]
    id: String,
    style: Style,
    handlers: Handlers<A>,
    fills_row: bool,
    fills_col: bool,
    content: NodeContent<Self, Text>,
}

/// Intermediate tree produced by the size pass, consumed by the position pass
pub struct SizedNode<A: App> {
    #[cfg(test)]
    id: String,
    style: Style,
    handlers: Handlers<A>,
    size: Size,
    content: NodeContent<Self, Text>,
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

    /// Entry point: three-pass layout (pre-process → size → position)
    pub(crate) fn layout(self, screen_w: u32, screen_h: u32) -> ResolvedNode<A> {
        let processed = self.pre_process(None.unwrap_or_default());
        let sized = processed.size_pass(screen_w, screen_h);
        sized.position_pass(0, 0)
    }

    /// Pass 0: resolve font inheritance, marshal text, and compute effective fill info.
    fn pre_process(self, parent_font: &'static Font) -> ProcessedNode<A> {
        let font = self.style.font.unwrap_or(parent_font);

        match self.content {
            NodeContent::Text(text_str) => {
                let fills_row = matches!(self.style.w, Sizing::Fill);
                let fills_col = matches!(self.style.h, Sizing::Fill);
                let text = Text::new(font, &text_str)
                    .anchor(crate::AnchorBox::Highlight, crate::AnchorCorner::TopLeft)
                    .position(0.0, 0.0);
                ProcessedNode {
                    #[cfg(test)]
                    id: self.id,
                    style: self.style,
                    handlers: self.handlers,
                    fills_row,
                    fills_col,
                    content: NodeContent::Text(text),
                }
            }
            NodeContent::Children(children) => {
                let processed_children: Vec<ProcessedNode<A>> =
                    children.into_iter().map(|c| c.pre_process(font)).collect();

                let fills_row = match self.style.w {
                    Sizing::Fill => true,
                    Sizing::Shrink => processed_children.iter().any(|c| c.fills_row),
                    Sizing::Fixed(_) => false,
                };
                let fills_col = match self.style.h {
                    Sizing::Fill => true,
                    Sizing::Shrink => processed_children.iter().any(|c| c.fills_col),
                    Sizing::Fixed(_) => false,
                };

                ProcessedNode {
                    #[cfg(test)]
                    id: self.id,
                    style: self.style,
                    handlers: self.handlers,
                    fills_row,
                    fills_col,
                    content: NodeContent::Children(processed_children),
                }
            }
        }
    }
}

impl<A: App> ProcessedNode<A> {
    fn effectively_fills(&self, axis: Axis) -> bool {
        match axis {
            Axis::Row => self.fills_row,
            Axis::Column => self.fills_col,
        }
    }

    /// Pass 1: compute sizes recursively. Fill nodes receive their share from the parent.
    fn size_pass(self, available_w: u32, available_h: u32) -> SizedNode<A> {
        match self.content {
            NodeContent::Text(text) => {
                let padding = self.style.padding.unwrap_or(0);
                let text_rect = text.rect();
                let intrinsic_w = text_rect.width() + padding * 2;
                let intrinsic_h = text_rect.height() + padding * 2;
                let w = match self.style.w {
                    Sizing::Fixed(w) => w,
                    Sizing::Fill => available_w.max(intrinsic_w),
                    Sizing::Shrink => intrinsic_w,
                };
                let h = match self.style.h {
                    Sizing::Fixed(h) => h,
                    Sizing::Fill => available_h.max(intrinsic_h),
                    Sizing::Shrink => intrinsic_h,
                };
                SizedNode {
                    #[cfg(test)]
                    id: self.id,
                    style: self.style,
                    handlers: self.handlers,
                    size: Size { width: w, height: h },
                    content: NodeContent::Text(text),
                }
            }
            NodeContent::Children(children) => {
                let padding = self.style.padding.unwrap_or(0);
                let axis = self.style.axis;
                let gap = self.style.gap.unwrap_or(0);

                let own_w = match self.style.w {
                    Sizing::Fixed(w) => Some(w),
                    Sizing::Fill => Some(available_w),
                    Sizing::Shrink => None,
                };
                let own_h = match self.style.h {
                    Sizing::Fixed(h) => Some(h),
                    Sizing::Fill => Some(available_h),
                    Sizing::Shrink => None,
                };

                let content_w = own_w.unwrap_or(available_w).saturating_sub(padding * 2);
                let content_h = own_h.unwrap_or(available_h).saturating_sub(padding * 2);

                // Partition children: separate fill-along-main from non-fill
                enum Slot<A: App> {
                    Sized(SizedNode<A>),
                    Deferred(ProcessedNode<A>),
                }

                let mut slots: Vec<Slot<A>> = Vec::with_capacity(children.len());
                let mut fill_count: u32 = 0;
                let mut consumed_main: u32 = 0;
                let mut flow_count: u32 = 0;

                for child in children {
                    let is_out_of_flow = !matches!(child.style.position, Position::Flow);
                    let fills_main = child.effectively_fills(axis);

                    if is_out_of_flow || fills_main {
                        if !is_out_of_flow {
                            fill_count += 1;
                            flow_count += 1;
                        }
                        slots.push(Slot::Deferred(child));
                    } else {
                        let sized = child.size_pass(content_w, content_h);
                        let main_size = match axis {
                            Axis::Row => sized.size.width,
                            Axis::Column => sized.size.height,
                        };
                        consumed_main += main_size;
                        flow_count += 1;
                        slots.push(Slot::Sized(sized));
                    }
                }

                // Compute fill budget with overflow redistribution.
                let total_gap = if flow_count > 1 {
                    gap * (flow_count - 1)
                } else {
                    0
                };
                let main_budget = match axis {
                    Axis::Row => content_w,
                    Axis::Column => content_h,
                };
                let remaining = main_budget
                    .saturating_sub(consumed_main)
                    .saturating_sub(total_gap);
                let fill_share = if fill_count > 0 {
                    remaining / fill_count
                } else {
                    0
                };

                // Resolve all deferred children
                let mut sized_children: Vec<SizedNode<A>> = Vec::with_capacity(slots.len());
                let mut fill_indices: Vec<usize> = Vec::new();

                for slot in slots {
                    let idx = sized_children.len();
                    match slot {
                        Slot::Sized(s) => sized_children.push(s),
                        Slot::Deferred(child) => {
                            let is_out_of_flow = !matches!(child.style.position, Position::Flow);
                            if is_out_of_flow {
                                sized_children.push(child.size_pass(available_w, available_h));
                            } else {
                                let (child_w, child_h) = match axis {
                                    Axis::Row => (fill_share, content_h),
                                    Axis::Column => (content_w, fill_share),
                                };
                                sized_children.push(child.size_pass(child_w, child_h));
                                fill_indices.push(idx);
                            }
                        }
                    }
                }

                // Handle overflow: if any fill child exceeds its share, shrink siblings
                let mut overflow_consumed: u32 = 0;
                let mut overflow_count: u32 = 0;
                for &i in &fill_indices {
                    let actual = match axis {
                        Axis::Row => sized_children[i].size.width,
                        Axis::Column => sized_children[i].size.height,
                    };
                    if actual > fill_share {
                        overflow_consumed += actual;
                        overflow_count += 1;
                    }
                }

                if overflow_count > 0 && overflow_count < fill_count {
                    let new_remaining = remaining.saturating_sub(overflow_consumed);
                    let new_share = new_remaining / (fill_count - overflow_count);
                    for &i in &fill_indices {
                        let actual = match axis {
                            Axis::Row => sized_children[i].size.width,
                            Axis::Column => sized_children[i].size.height,
                        };
                        if actual <= fill_share {
                            match axis {
                                Axis::Row => sized_children[i].size.width = new_share,
                                Axis::Column => sized_children[i].size.height = new_share,
                            }
                        }
                    }
                }

                // Compute final content size
                let mut max_cross: u32 = 0;
                let mut total_main: u32 = 0;
                let mut final_flow_count: u32 = 0;

                for sized in &sized_children {
                    let is_out_of_flow = !matches!(sized.style.position, Position::Flow);
                    if !is_out_of_flow {
                        let (main, cross) = match axis {
                            Axis::Row => (sized.size.width, sized.size.height),
                            Axis::Column => (sized.size.height, sized.size.width),
                        };
                        total_main += main;
                        max_cross = max_cross.max(cross);
                        final_flow_count += 1;
                    }
                }

                let total_main_with_gaps = if final_flow_count > 1 {
                    total_main + gap * (final_flow_count - 1)
                } else {
                    total_main
                };

                let content_total_w = (match axis {
                    Axis::Row => total_main_with_gaps,
                    Axis::Column => max_cross,
                }) + padding * 2;
                let content_total_h = (match axis {
                    Axis::Row => max_cross,
                    Axis::Column => total_main_with_gaps,
                }) + padding * 2;

                let final_w = match self.style.w {
                    Sizing::Fixed(w) => w,
                    Sizing::Shrink => content_total_w,
                    Sizing::Fill => content_total_w.max(available_w),
                };
                let final_h = match self.style.h {
                    Sizing::Fixed(h) => h,
                    Sizing::Shrink => content_total_h,
                    Sizing::Fill => content_total_h.max(available_h),
                };

                SizedNode {
                    #[cfg(test)]
                    id: self.id,
                    style: self.style,
                    handlers: self.handlers,
                    size: Size {
                        width: final_w,
                        height: final_h,
                    },
                    content: NodeContent::Children(sized_children),
                }
            }
        }
    }
}

impl<A: App> SizedNode<A> {
    /// Pass 2: assign positions to produce the final ResolvedNode tree.
    fn position_pass(self, origin_x: i32, origin_y: i32) -> ResolvedNode<A> {
        let (x, y) = match self.style.position {
            Position::Flow => (origin_x, origin_y),
            Position::Relative(rx, ry) => (origin_x + rx, origin_y + ry),
            Position::Absolute(ax, ay) => (ax, ay),
        };

        match self.content {
            NodeContent::Text(text) => {
                let padding = self.style.padding.unwrap_or(0);
                let text_rect = text.rect();
                let content_w = self.size.width.saturating_sub(padding * 2);
                let content_h = self.size.height.saturating_sub(padding * 2);
                let text_w = text_rect.width();
                let text_h = text_rect.height();

                let text_offset_x = match self.style.justify {
                    Justify::Start => 0,
                    Justify::Center | Justify::SpaceBetween => {
                        (content_w as i32 - text_w as i32) / 2
                    }
                    Justify::End => content_w as i32 - text_w as i32,
                };
                let text_offset_y = match self.style.align {
                    Align::Start => 0,
                    Align::Center => (content_h as i32 - text_h as i32) / 2,
                    Align::End => content_h as i32 - text_h as i32,
                };

                let text_x = x + padding as i32 + text_offset_x;
                let text_y = y + padding as i32 + text_offset_y;
                let text = text.position(text_x as f32, text_y as f32);
                let rect =
                    Rect::from_top_left(x as f32, y as f32, self.size.width, self.size.height);
                ResolvedNode {
                    #[cfg(test)]
                    id: self.id,
                    rect,
                    style: self.style,
                    content: NodeContent::Text(text),
                    handlers: self.handlers,
                }
            }
            NodeContent::Children(children) => {
                let padding = self.style.padding.unwrap_or(0);
                let axis = self.style.axis;
                let gap = self.style.gap.unwrap_or(0);
                let justify = self.style.justify;
                let align = self.style.align;
                let content_x = x + padding as i32;
                let content_y = y + padding as i32;
                let content_w = self.size.width.saturating_sub(padding * 2);
                let content_h = self.size.height.saturating_sub(padding * 2);

                let flow_children: Vec<&SizedNode<A>> = children
                    .iter()
                    .filter(|c| matches!(c.style.position, Position::Flow))
                    .collect();
                let flow_count = flow_children.len() as u32;

                let total_main: u32 = flow_children
                    .iter()
                    .map(|c| match axis {
                        Axis::Row => c.size.width,
                        Axis::Column => c.size.height,
                    })
                    .sum();
                let total_gaps = if flow_count > 1 {
                    gap * (flow_count - 1)
                } else {
                    0
                };
                let main_budget = match axis {
                    Axis::Row => content_w,
                    Axis::Column => content_h,
                };
                let leftover = main_budget as i32 - total_main as i32 - total_gaps as i32;

                let (mut main_cursor, justify_gap) = match justify {
                    Justify::Start => (0i32, 0i32),
                    Justify::Center => (leftover / 2, 0),
                    Justify::End => (leftover, 0),
                    Justify::SpaceBetween => {
                        if flow_count > 1 {
                            (0, leftover / (flow_count as i32 - 1))
                        } else {
                            (0, 0)
                        }
                    }
                };

                let cross_budget = match axis {
                    Axis::Row => content_h,
                    Axis::Column => content_w,
                };

                let mut resolved_children = Vec::new();

                for child in children {
                    let is_out_of_flow = !matches!(child.style.position, Position::Flow);

                    let child_cross = match axis {
                        Axis::Row => child.size.height,
                        Axis::Column => child.size.width,
                    };
                    let cross_offset = if is_out_of_flow {
                        0
                    } else {
                        match align {
                            Align::Start => 0,
                            Align::Center => (cross_budget as i32 - child_cross as i32) / 2,
                            Align::End => cross_budget as i32 - child_cross as i32,
                        }
                    };

                    let (child_origin_x, child_origin_y) = match axis {
                        Axis::Row => (content_x + main_cursor, content_y + cross_offset),
                        Axis::Column => (content_x + cross_offset, content_y + main_cursor),
                    };
                    let resolved = child.position_pass(child_origin_x, child_origin_y);

                    if !is_out_of_flow {
                        let child_main = match axis {
                            Axis::Row => resolved.rect.width(),
                            Axis::Column => resolved.rect.height(),
                        };
                        main_cursor += child_main as i32 + gap as i32 + justify_gap;
                    }

                    resolved_children.push(resolved);
                }

                let rect =
                    Rect::from_top_left(x as f32, y as f32, self.size.width, self.size.height);
                ResolvedNode {
                    #[cfg(test)]
                    id: self.id,
                    rect,
                    style: self.style,
                    content: NodeContent::Children(resolved_children),
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

pub(crate) struct ResolvedNode<A: App> {
    #[cfg(test)]
    id: String,
    rect: Rect,
    style: Style,
    content: NodeContent<Self, Text>,
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
            self.style
                .pressed_color
                .or(self.style.hover_color)
                .or(self.style.color)
        } else if hit.is_hovered() {
            self.style.hover_color.or(self.style.color)
        } else {
            self.style.color
        };

        let text_color = if hit.is_down() {
            self.style
                .pressed_text_color
                .or(self.style.hover_text_color)
                .or(self.style.text_color)
        } else if hit.is_hovered() {
            self.style.hover_text_color.or(self.style.text_color)
        } else {
            self.style.text_color
        }
        .or(text_color);

        let text = match self.content {
            NodeContent::Children(children) => {
                for node in children.into_iter().rev() {
                    node.evaluate_recursive(app, state, cells, consumed, text_color, depth + 1.0);
                }
                None
            }
            NodeContent::Text(text) => Some(text),
        };

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
        if let (Some(text), Some(text_color)) = (text, text_color) {
            text.color(text_color).emit_cells(&mut |mut c| {
                c.position.z = depth + 0.5;
                cells.push(c);
            });
        }
    }

    #[cfg(test)]
    pub(crate) fn find_child_by_index(&self, index: usize) -> Option<&Self> {
        match &self.content {
            NodeContent::Children(children) => children.get(index),
            NodeContent::Text(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn find_child_by_id(&self, id: &str) -> Option<&Self> {
        match &self.content {
            NodeContent::Children(children) => {
                children.iter().find_map(|c| (c.id == id).then_some(c))
            }
            NodeContent::Text(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn text_rect(&self) -> Option<Rect> {
        match &self.content {
            NodeContent::Text(t) => Some(t.rect()),
            NodeContent::Children(_) => None,
        }
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
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 0.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.y(), 5.0);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.y(), 10.0);
    }

    #[test]
    fn row_layout_stacks_horizontally() {
        let node: Node<TestApp> = row().children(vec![row().size(10, 5), row().size(10, 5)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 20);
        assert_eq!(resolved.rect.height(), 5);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 0.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 10.0);
    }

    #[test]
    fn gap_between_children() {
        let node: Node<TestApp> = row()
            .gap(4)
            .children(vec![row().size(10, 5), row().size(10, 5)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 24);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 14.0);
    }

    #[test]
    fn padding_offsets_children() {
        let node: Node<TestApp> = col().padding(3).children(vec![row().size(4, 4)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.rect.width(), 10); // 4 + 3*2
        assert_eq!(resolved.rect.height(), 10);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 3.0);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 3.0);
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
        let child = resolved.find_child_by_index(0).unwrap();
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
        let child = resolved.find_child_by_index(0).unwrap();
        assert!(child.rect.width() > 0);
        assert!(child.rect.height() > 0);
    }

    #[test]
    fn text_node_with_padding() {
        let node: Node<TestApp> = col().children(vec![text("A").padding(3)]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let no_pad: Node<TestApp> = col().children(vec![text("A")]);
        let no_pad_resolved = no_pad.layout(256, 256);
        let no_pad_child = no_pad_resolved.find_child_by_index(0).unwrap();
        assert_eq!(child.rect.width(), no_pad_child.rect.width() + 6);
        assert_eq!(child.rect.height(), no_pad_child.rect.height() + 6);
    }

    // #[test]
    // fn text_node_omits_uncolored_cells() {
    //     let node: Node<TestApp> = col().children(vec![text("I")]);
    //     let mut app = TestApp::new();
    //     let mut state = make_state();
    //     state.set_input(input_at(100.0, 100.0));
    //     let (cells, _) = node.layout(256, 256).evaluate(&mut app, &mut state);
    //     assert!(cells.is_empty());
    // }

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
        match &resolved.content {
            NodeContent::Children(children) => match &children[0].content {
                NodeContent::Text(t) => assert!(std::ptr::eq(t.font(), &TOM_THUMB_3X5)),
                _ => panic!("expected text node"),
            },
            _ => panic!("expected children"),
        }
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

    // --- Fill distribution tests ---

    #[test]
    fn fill_children_share_space_equally() {
        let node: Node<TestApp> = row()
            .width(100)
            .children(vec![pane().fill_w().height(10), pane().fill_w().height(10)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 50);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 50);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 50.0);
    }

    #[test]
    fn fill_children_share_remaining_after_fixed() {
        let node: Node<TestApp> = row().width(100).children(vec![
            pane().size(20, 10),
            pane().fill_w().height(10),
            pane().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        // 100 - 20 fixed = 80 remaining, split equally = 40 each
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 20);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 40);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.width(), 40);
    }

    #[test]
    fn fill_with_gap_accounts_for_gaps() {
        let node: Node<TestApp> = row()
            .width(100)
            .gap(10)
            .children(vec![pane().fill_w().height(10), pane().fill_w().height(10)]);
        let resolved = node.layout(256, 256);
        // 100 - 10 gap = 90 remaining, split = 45 each
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 45);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 45);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 55.0); // 45 + 10 gap
    }

    #[test]
    fn fill_in_column() {
        let node: Node<TestApp> = col()
            .height(60)
            .children(vec![pane().size(10, 20), pane().fill_h().width(10)]);
        let resolved = node.layout(256, 256);
        // 60 - 20 fixed = 40 for fill child
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.height(), 40);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.y(), 20.0);
    }

    #[test]
    fn three_fill_children_equal() {
        let node: Node<TestApp> = row().width(90).children(vec![
            pane().fill_w().height(10),
            pane().fill_w().height(10),
            pane().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 30);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 30);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.width(), 30);
    }

    #[test]
    fn fill_overflow_shrinks_siblings() {
        // Child with fixed content wider than equal share forces siblings smaller
        let node: Node<TestApp> = row().width(100).children(vec![
            pane().fill_w().height(10).children(vec![
                pane().size(60, 10), // content forces 60, equal share would be 50
            ]),
            pane().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        // First child overflows to 60, second gets remaining 40
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 60);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 40);
    }

    #[test]
    fn fill_overflow_multiple_siblings() {
        let node: Node<TestApp> = row().width(120).children(vec![
            pane().fill_w().height(10).children(vec![
                pane().size(80, 10), // overflows the 40 equal share
            ]),
            pane().fill_w().height(10),
            pane().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        // First overflows to 80, remaining 40 split equally = 20 each
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 80);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 20);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.width(), 20);
    }

    #[test]
    fn fill_single_nested_fill() {
        let node: Node<TestApp> = row().width(120).children(vec![
            row()
                .height(10)
                .children(vec![row().fill_w(), row().fill_w()]),
            row().fill_w().height(10),
            row().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 40);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 40);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.width(), 40);
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(0)
                .unwrap()
                .rect
                .width(),
            20
        );
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(1)
                .unwrap()
                .rect
                .width(),
            20
        );
    }

    #[test]
    fn fill_double_nested_fill() {
        let node: Node<TestApp> = row().width(120).children(vec![
            row().height(10).children(vec![
                row().width(10),
                row()
                    .height(10)
                    .children(vec![row().fill_w(), row().fill_w()]),
                row().width(10),
            ]),
            row().fill_w().height(10),
            row().fill_w().height(10),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 40);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 40);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.width(), 40);
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(0)
                .unwrap()
                .rect
                .width(),
            10
        );
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(1)
                .unwrap()
                .rect
                .width(),
            20
        );
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(2)
                .unwrap()
                .rect
                .width(),
            10
        );
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(1)
                .unwrap()
                .find_child_by_index(0)
                .unwrap()
                .rect
                .width(),
            10
        );
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(1)
                .unwrap()
                .find_child_by_index(1)
                .unwrap()
                .rect
                .width(),
            10
        );
    }

    #[test]
    fn fill_cross_axis_in_row() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .children(vec![pane().width(50).fill_h(), pane().width(50).height(30)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.height(), 60);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.height(), 30);
    }

    #[test]
    fn fill_cross_axis_in_column() {
        let node: Node<TestApp> = col().size(60, 100).children(vec![
            pane().fill_w().height(50),
            pane().width(30).height(50),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 60);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.width(), 30);
    }

    #[test]
    fn fill_cross_axis_nested_in_row() {
        let node: Node<TestApp> = row().size(100, 60).children(vec![
            pane().width(50).children(vec![pane().fill_h()]),
            pane().width(50).height(30),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.height(), 60);
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(0)
                .unwrap()
                .rect
                .height(),
            60
        );
    }

    #[test]
    fn fill_cross_axis_nested_in_column() {
        let node: Node<TestApp> = col().size(60, 100).children(vec![
            pane().height(50).children(vec![pane().fill_w()]),
            pane().width(30).height(50),
        ]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.width(), 60);
        assert_eq!(
            resolved
                .find_child_by_index(0)
                .unwrap()
                .find_child_by_index(0)
                .unwrap()
                .rect
                .width(),
            60
        );
    }

    // --- Justify tests ---

    #[test]
    fn justify_start_is_default() {
        let node: Node<TestApp> = row()
            .width(100)
            .children(vec![pane().size(20, 10), pane().size(20, 10)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 0.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 20.0);
    }

    #[test]
    fn justify_center_row() {
        let node: Node<TestApp> = row()
            .width(100)
            .justify_center()
            .children(vec![pane().size(20, 10), pane().size(20, 10)]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 40 = 60, offset = 30
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 30.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 50.0);
    }

    #[test]
    fn justify_center_column() {
        let node: Node<TestApp> = col()
            .height(100)
            .justify_center()
            .children(vec![pane().size(10, 20), pane().size(10, 20)]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 40 = 60, offset = 30
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 30.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.y(), 50.0);
    }

    #[test]
    fn justify_end_row() {
        let node: Node<TestApp> = row()
            .width(100)
            .justify_end()
            .children(vec![pane().size(20, 10), pane().size(20, 10)]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 40 = 60
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 60.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 80.0);
    }

    #[test]
    fn justify_end_column() {
        let node: Node<TestApp> = col()
            .height(100)
            .justify_end()
            .children(vec![pane().size(10, 20), pane().size(10, 20)]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 40 = 60
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 60.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.y(), 80.0);
    }

    #[test]
    fn justify_space_between_row() {
        let node: Node<TestApp> = row()
            .width(100)
            .justify_full()
            .children(vec![
                pane().size(20, 10),
                pane().size(20, 10),
                pane().size(20, 10),
            ]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 60 = 40, gap = 40 / 2 = 20
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 0.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 40.0);
        assert_eq!(resolved.find_child_by_index(2).unwrap().rect.x(), 80.0);
    }

    #[test]
    fn justify_space_between_single_child() {
        let node: Node<TestApp> = row()
            .width(100)
            .justify_full()
            .children(vec![pane().size(20, 10)]);
        let resolved = node.layout(256, 256);
        // single child: starts at 0
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 0.0);
    }

    #[test]
    fn justify_center_overflow() {
        let node: Node<TestApp> = row()
            .width(40)
            .justify_center()
            .children(vec![pane().size(30, 10), pane().size(30, 10)]);
        let resolved = node.layout(256, 256);
        // leftover = 40 - 60 = -20, offset = -10 (overflows equally)
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), -10.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 20.0);
    }

    #[test]
    fn justify_center_with_gap() {
        let node: Node<TestApp> = row()
            .width(100)
            .gap(10)
            .justify_center()
            .children(vec![pane().size(20, 10), pane().size(20, 10)]);
        let resolved = node.layout(256, 256);
        // leftover = 100 - 40 - 10 = 50, offset = 25
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 25.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.x(), 55.0);
    }

    // --- Align tests ---

    #[test]
    fn align_start_is_default() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 0.0);
    }

    #[test]
    fn align_center_row() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .align_center()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // cross = 60 - 20 = 40, offset = 20
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 20.0);
    }

    #[test]
    fn align_center_column() {
        let node: Node<TestApp> = col()
            .size(60, 100)
            .align_center()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // cross = 60 - 20 = 40, offset = 20
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 20.0);
    }

    #[test]
    fn align_end_row() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .align_end()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // cross = 60 - 20 = 40
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 40.0);
    }

    #[test]
    fn align_end_column() {
        let node: Node<TestApp> = col()
            .size(60, 100)
            .align_end()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // cross = 60 - 20 = 40
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 40.0);
    }

    #[test]
    fn align_center_multiple_children() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .align_center()
            .children(vec![pane().size(20, 20), pane().size(20, 40)]);
        let resolved = node.layout(256, 256);
        // each child centered independently
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 20.0);
        assert_eq!(resolved.find_child_by_index(1).unwrap().rect.y(), 10.0);
    }

    #[test]
    fn justify_and_align_combined() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .justify_center()
            .align_center()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // justify: leftover = 100 - 20 = 80, offset = 40
        // align: cross = 60 - 20 = 40, offset = 20
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 40.0);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 20.0);
    }

    #[test]
    fn justify_and_align_with_padding() {
        let node: Node<TestApp> = row()
            .size(100, 60)
            .padding(10)
            .justify_center()
            .align_center()
            .children(vec![pane().size(20, 20)]);
        let resolved = node.layout(256, 256);
        // content area: 80x40, justify offset = (80-20)/2 = 30, align offset = (40-20)/2 = 10
        // child at: padding(10) + offset
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.x(), 40.0);
        assert_eq!(resolved.find_child_by_index(0).unwrap().rect.y(), 20.0);
    }

    // --- Text alignment tests ---

    #[test]
    fn text_fixed_size_larger_than_intrinsic() {
        let node: Node<TestApp> = col().children(vec![text("A").size(50, 30)]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        assert_eq!(child.rect.width(), 50);
        assert_eq!(child.rect.height(), 30);
    }

    #[test]
    fn text_fill_w_takes_available() {
        let node: Node<TestApp> = row().width(100).children(vec![text("A").fill_w()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        assert_eq!(child.rect.width(), 100);
    }

    #[test]
    fn text_justify_start_default() {
        let node: Node<TestApp> = col().children(vec![text("A").width(50)]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        assert_eq!(tr.x(), 0.0);
    }

    #[test]
    fn text_justify_center() {
        let node: Node<TestApp> = col().children(vec![text("A").width(50).justify_center()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_w = tr.width();
        let expected_x = (50 - text_w) / 2;
        assert_eq!(tr.x(), expected_x as f32);
    }

    #[test]
    fn text_justify_end() {
        let node: Node<TestApp> = col().children(vec![text("A").width(50).justify_end()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_w = tr.width();
        let expected_x = 50 - text_w;
        assert_eq!(tr.x(), expected_x as f32);
    }

    #[test]
    fn text_align_center() {
        let node: Node<TestApp> = col().children(vec![text("A").size(50, 30).align_center()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_h = tr.height();
        let expected_y = (30 - text_h) / 2;
        assert_eq!(tr.y(), expected_y as f32);
    }

    #[test]
    fn text_align_end() {
        let node: Node<TestApp> = col().children(vec![text("A").size(50, 30).align_end()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_h = tr.height();
        let expected_y = 30 - text_h;
        assert_eq!(tr.y(), expected_y as f32);
    }

    #[test]
    fn text_justify_center_align_center() {
        let node: Node<TestApp> =
            col().children(vec![text("A").size(50, 30).justify_center().align_center()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_w = tr.width();
        let text_h = tr.height();
        assert_eq!(tr.x(), ((50 - text_w) / 2) as f32);
        assert_eq!(tr.y(), ((30 - text_h) / 2) as f32);
    }

    #[test]
    fn text_justify_center_with_padding() {
        let node: Node<TestApp> =
            col().children(vec![text("A").width(50).padding(5).justify_center()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        let tr = child.text_rect().unwrap();
        let text_w = tr.width();
        // content area = 50 - 10 = 40, offset = (40 - text_w) / 2, then shifted by padding
        let expected_x = 5 + (40 - text_w) / 2;
        assert_eq!(tr.x(), expected_x as f32);
    }

    #[test]
    fn text_positioned_in_parent_with_justify() {
        let node: Node<TestApp> = row()
            .width(100)
            .justify_center()
            .children(vec![text("A").width(40).justify_center()]);
        let resolved = node.layout(256, 256);
        let child = resolved.find_child_by_index(0).unwrap();
        // child rect centered in parent: (100-40)/2 = 30
        assert_eq!(child.rect.x(), 30.0);
        // text centered within child
        let tr = child.text_rect().unwrap();
        let text_w = tr.width();
        let expected_x = 30 + (40 - text_w) / 2;
        assert_eq!(tr.x(), expected_x as f32);
    }
}
