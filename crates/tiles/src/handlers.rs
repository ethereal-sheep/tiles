use crate::__private::WidgetFn;
use crate::element::DragInfo;
use crate::{Node, Props};
use tiles_macros::Builders;
// --- Handlers ---

#[derive(Builders, Default)]
#[builders(forward(to = "Node", via = "handlers"))]
#[builders(forward(to = "WidgetFn<F: FnOnce(Props) -> Node>", via = "handlers"))]
pub struct Handlers {
    pub on_hover: Option<Box<dyn Fn()>>,
    pub on_enter: Option<Box<dyn Fn()>>,
    pub on_leave: Option<Box<dyn Fn()>>,
    pub on_click: Option<Box<dyn Fn()>>,
    pub on_double_click: Option<Box<dyn Fn()>>,
    pub on_press: Option<Box<dyn Fn()>>,
    pub on_release: Option<Box<dyn Fn()>>,
    pub on_right_click: Option<Box<dyn Fn()>>,
    pub on_hold: Option<Box<dyn Fn()>>,
    pub on_drag: Option<Box<dyn Fn(DragInfo)>>,
    pub on_drag_end: Option<Box<dyn Fn(DragInfo)>>,
    pub on_scroll: Option<Box<dyn Fn(f32)>>,
}
