mod camera;
mod cell;
mod color;
mod config;
mod drawable;
mod element;
pub mod font;
mod input;
mod line;
mod rect;
#[cfg(feature = "runtime")]
mod renderer;
#[cfg(feature = "runtime")]
mod runner;
mod shape;
mod text;
pub mod ui;

pub use camera::Camera;
pub use cell::{Cell, Rotation};
pub use color::Color;
pub use config::{Config, ConfigBuilder};
pub use drawable::{Drawable, Transformable};
pub use element::{DragInfo, Element, ElementState, HitState};
pub use input::{KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent};
pub use line::Line;
pub use rect::{Rect, RoundedRect};
#[cfg(feature = "runtime")]
pub use runner::{App, State};
pub use shape::{Fill, Shape, Stroke, StrokePosition};
pub use text::{AnchorBox, AnchorCorner, Text};
pub use tiles_macros::ui;
pub use ui::{text, Node};

#[cfg(feature = "runtime")]
pub fn run(
    mut app: impl App + 'static,
    config: Config,
) -> Result<(), winit::error::EventLoopError> {
    runner::run_app(&mut app, config)
}
