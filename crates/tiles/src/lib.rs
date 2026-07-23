extern crate self as tiles;

mod anchor;
mod camera;
mod cell;
mod color;
mod config;
#[cfg(feature = "runtime")]
mod context;
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
    pub mod blocks {
        pub use crate::node::{col, img, paint, pane, row, text};
    }
    pub mod macros {
        pub use tiles_macros::{widget, widget_fn};
    }
    pub mod hooks {
        pub use crate::context::{get_app, get_state};
        pub use crate::signal::signal;
    }
}

pub use cell::{Cell, Rotation};
pub use color::Color;
pub use config::{Config, ConfigBuilder};
#[cfg(feature = "runtime")]
pub use context::{AppContext, StateContext};
pub use drawable::Drawable;
pub use element::{DragInfo, Element, ElementState, HitState};
pub use image::{Frame, Image, ImageError, Sprite};
pub use input::{KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent};
pub use line::Line;
pub use node::{Handlers, Node, Props};
pub use rect::{Rect, RoundedRect};
#[cfg(feature = "runtime")]
pub use runner::{App, State};
pub use shape::{Fill, Shape, Stroke, StrokePosition};
pub use size::Size;
pub use style::Style;
pub use text::Text;

#[doc(hidden)]
pub mod __private {
    pub use crate::node::{BlankWidgetFn, Node, Widget, WidgetFn};
    pub use crate::signal::{__pop_widget, __push_widget, __widget_id};
}

#[cfg(feature = "runtime")]
pub fn run(
    mut app: impl App + 'static,
    config: Config,
) -> Result<(), winit::error::EventLoopError> {
    runner::run_app(&mut app, config)
}
