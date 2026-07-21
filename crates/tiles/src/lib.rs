extern crate self as tiles;

mod anchor;
mod camera;
mod cell;
mod color;
mod config;
mod drawable;
mod element;
pub mod font;
mod image;
mod input;
mod line;
mod node;
mod rect;
#[cfg(feature = "runtime")]
mod renderer;
#[cfg(feature = "runtime")]
mod runner;
mod shape;
pub mod signal;
mod size;
mod style;
mod text;
pub mod ui {
    pub use crate::node::{col, img, paint, pane, row, text};
    pub use crate::signal::signal;
    pub use tiles_macros::{app_widget, new_widget_fn, widget, widget_fn};
}

pub use cell::{Cell, Rotation};
pub use color::Color;
pub use config::{Config, ConfigBuilder};
pub use drawable::Drawable;
pub use element::{DragInfo, Element, ElementState, HitState};
pub use image::{Frame, Image, ImageError, Sprite};
pub use input::{KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent};
pub use line::Line;
pub use node::{Node, NodeData};
pub use rect::{Rect, RoundedRect};
#[cfg(feature = "runtime")]
pub use runner::{App, State};
pub use shape::{Fill, Shape, Stroke, StrokePosition};
pub use size::Size;
pub use style::Style;
pub use text::Text;

#[doc(hidden)]
pub mod __private {
    pub use crate::node::{NewWidgetFn, Node, Widget, WidgetFn};
    pub use crate::signal::{__pop_widget, __push_widget, __widget_id};
}

#[cfg(feature = "runtime")]
pub fn run(
    mut app: impl App + 'static,
    config: Config,
) -> Result<(), winit::error::EventLoopError> {
    runner::run_app(&mut app, config)
}
