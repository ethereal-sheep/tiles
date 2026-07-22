use crate::__private::NewWidgetFn;
use crate::color::Color;
use crate::font::Font;
use crate::{Node, NodeData};
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
    Relative(f32, f32),
    Absolute(f32, f32),
}

// --- Styles ---

// 1. auto mark field as required builder unless marked as custom, or marked as omit
// 2. mark field as inheritable, and generate a function which allows a Style to inherit from another style
//      if is None

#[derive(Clone, Debug, Default, Builders)]
#[builders(forward(to = "Node", via = "style"))]
#[builders(forward(
    to = "NewWidgetFn<F: FnOnce(NodeData) -> Node>",
    via = "style"
))]
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
    pub gap: u32,
    pub padding: u32,
    #[builder(variant(name = "justify_start", variant = "Start"))]
    #[builder(variant(name = "justify_center", variant = "Center"))]
    #[builder(variant(name = "justify_end", variant = "End"))]
    #[builder(variant(name = "justify_full", variant = "SpaceBetween"))]
    pub justify: Justify,
    #[builder(variant(name = "align_start", variant = "Start"))]
    #[builder(variant(name = "align_center", variant = "Center"))]
    #[builder(variant(name = "align_end", variant = "End"))]
    pub align: Align,
    #[builder(variant(name = "relative", variant = "Relative", args = "x: f32, y: f32"))]
    #[builder(variant(name = "absolute", variant = "Absolute", args = "x: f32, y: f32"))]
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
