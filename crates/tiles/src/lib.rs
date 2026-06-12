mod camera;
mod cell;
mod color;
mod config;
mod drawable;
pub mod font;
mod input;
mod line;
mod rect;
mod renderer;
mod runner;
mod shape;
mod text;

pub use camera::Camera;
pub use cell::{Cell, Rotation};
pub use color::Color;
pub use config::{Config, ConfigBuilder};
pub use drawable::{Colored, Drawable};
pub use input::{KeyCode, KeyState, MouseAction, MouseButton, MouseEvent, KeyEvent, RectInputState};
pub use line::Line;
pub use rect::{Rect, RoundedRect};
pub use runner::{App, State};
pub use shape::{Fill, Shape, Stroke, StrokePosition};
pub use text::{AnchorBox, AnchorCorner, Text};

pub fn run(mut app: impl App + 'static, config: Config) -> Result<(), winit::error::EventLoopError> {
    runner::run_app(&mut app, config)
}
