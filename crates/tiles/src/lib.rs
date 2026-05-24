mod camera;
mod cell;
mod config;
pub mod font;
mod input;
mod renderer;
mod runner;

pub use camera::Camera;
pub use cell::{Cell, Rotation};
pub use config::{Config, ConfigBuilder};
pub use input::{KeyCode, KeyState, MouseButton, MouseEvent, KeyEvent};
pub use runner::{App, State};

pub fn run(mut app: impl App + 'static, config: Config) -> Result<(), winit::error::EventLoopError> {
    runner::run_app(&mut app, config)
}
