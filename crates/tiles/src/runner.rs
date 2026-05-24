use glam::Vec2;
use pollster::block_on;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::camera::Camera;
use crate::cell::{Cell, CellInstance, LightData};
use crate::config::Config;
use crate::input::{self, InputState, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent};
use crate::renderer::Renderer;

pub trait App {
    fn init(&mut self, _state: &mut State) {}
    fn update(&mut self, _state: &mut State) {}
    fn draw(&mut self, _state: &mut State) {}
    fn on_key(&mut self, _state: &mut State, _event: KeyEvent) {}
    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

pub struct State {
    window: Option<std::sync::Arc<Window>>,
    renderer: Option<Renderer>,
    cells: Vec<Cell>,
    config: Config,
    camera: Camera,
    input: InputState,
    pub dt: f32,
    pub elapsed: f32,
    pub alpha: f32,
    pub quit: bool,
    timer: Option<Instant>,
    start: Option<Instant>,
    accumulator: f32,
    fixed_dt: f32,
    debug: bool,
    window_bg: [f32; 4],
    viewport_bg: [f32; 4],
    ambient_illumination: f32,
}

impl State {
    fn new(config: Config) -> Self {
        let fixed_dt = 1.0 / config.steps_per_second as f32;
        let camera = Camera::new(config.viewport_width, config.viewport_height);
        Self {
            window: None,
            renderer: None,
            cells: Vec::new(),
            config,
            camera,
            input: InputState::new(),
            dt: fixed_dt,
            elapsed: 0.0,
            alpha: 0.0,
            quit: false,
            timer: None,
            start: None,
            accumulator: 0.0,
            fixed_dt,
            debug: false,
            window_bg: [0.0, 0.0, 0.0, 1.0],
            viewport_bg: [0.08, 0.08, 0.10, 1.0],
            ambient_illumination: 1.0,
        }
    }

    // --- Drawing ---

    pub fn draw(&mut self, cell: Cell) {
        self.cells.push(cell);
    }

    pub fn draw_all(&mut self, cells: &[Cell]) {
        self.cells.extend(cells.iter().map(|c| Cell {
            position: c.position,
            color: c.color,
            quat: c.quat,
            light_radius: c.light_radius,
            intensity: c.intensity,
        }));
    }

    // --- Camera ---

    pub fn set_camera_position(&mut self, x: f32, y: f32) {
        self.camera.position = Vec2::new(x, y);
    }

    pub fn camera_position(&self) -> Vec2 {
        self.camera.position
    }

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.camera.viewport_width = width;
        self.camera.viewport_height = height;
        self.config.viewport_width = width;
        self.config.viewport_height = height;
        self.config.save();
    }

    pub fn viewport_size(&self) -> Vec2 {
        Vec2::new(self.camera.viewport_width, self.camera.viewport_height)
    }

    // --- Window ---

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if let Some(window) = &self.window {
            let _ = window.request_inner_size(winit::dpi::LogicalSize::new(width, height));
        }
        self.config.save();
    }

    pub fn set_title(&mut self, title: &str) {
        self.config.title = title.to_owned();
        if let Some(window) = &self.window {
            window.set_title(title);
        }
        self.config.save();
    }

    pub fn set_window_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.window_bg = [r, g, b, a];
    }

    pub fn set_viewport_background(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.viewport_bg = [r, g, b, a];
    }

    pub fn set_ambient_illumination(&mut self, ambient: f32) {
        self.ambient_illumination = ambient.clamp(0.0, 1.0);
    }

    // --- Timing ---

    pub fn set_steps_per_second(&mut self, steps: u32) {
        self.fixed_dt = 1.0 / steps as f32;
        self.config.steps_per_second = steps;
        self.config.save();
    }

    // --- Input (polled) ---

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.input.keys_down.contains(&key)
    }

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.input.mouse_buttons_down.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.input.mouse_world_pos
    }

    pub fn mouse_screen_position(&self) -> Vec2 {
        self.input.mouse_screen_pos
    }

    // --- Debug ---

    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }

    pub fn debug_line(&mut self, _from: Vec2, _to: Vec2, _color: [f32; 4]) {
        if !self.debug {
            return;
        }
        // TODO: debug line rendering
    }

    pub fn debug_text(&mut self, _text: &str, _x: f32, _y: f32) {
        if !self.debug {
            return;
        }
        // TODO: debug text rendering with bitmap font
    }

    // --- Config ---

    pub fn set_vsync(&mut self, vsync: bool) {
        self.config.vsync = vsync;
        if let Some(renderer) = &mut self.renderer {
            renderer.set_vsync(vsync);
        }
        self.config.save();
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.config.resizable = resizable;
        self.config.save();
    }
}

pub(crate) fn run_app(app: &mut (impl App + 'static), config: Config) -> Result<(), winit::error::EventLoopError> {
    let state = State::new(config);
    let event_loop = EventLoop::new().unwrap();

    // Safety: we need 'static for the runner but app lives for the duration of run
    let app_ptr = app as *mut dyn App;
    let mut runner = Runner {
        app: unsafe { &mut *app_ptr },
        state,
    };
    event_loop.run_app(&mut runner)
}

struct Runner<'a> {
    app: &'a mut dyn App,
    state: State,
}

impl ApplicationHandler for Runner<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title(&self.state.config.title)
            .with_resizable(self.state.config.resizable)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.state.config.width,
                self.state.config.height,
            ));

        let window = std::sync::Arc::new(event_loop.create_window(attrs).unwrap());
        let mut renderer = block_on(Renderer::new(window.clone()));
        if !self.state.config.vsync {
            renderer.set_vsync(false);
        }
        window.request_redraw();
        self.state.window = Some(window);
        self.state.renderer = Some(renderer);
        self.state.start = Some(Instant::now());
        self.state.timer = Some(Instant::now());
        self.app.init(&mut self.state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        if self.state.quit {
            event_loop.exit();
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                let key = input::translate_key(event.physical_key);
                let key_state = match event.state {
                    ElementState::Pressed => KeyState::Pressed,
                    ElementState::Released => KeyState::Released,
                };

                match key_state {
                    KeyState::Pressed => { self.state.input.keys_down.insert(key); }
                    KeyState::Released => { self.state.input.keys_down.remove(&key); }
                }

                self.app.on_key(&mut self.state, KeyEvent { key, state: key_state });
            }

            WindowEvent::MouseInput { state: btn_state, button, .. } => {
                let mb = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    _ => return,
                };

                let event = match btn_state {
                    ElementState::Pressed => {
                        self.state.input.mouse_buttons_down.insert(mb);
                        MouseEvent::Pressed(mb)
                    }
                    ElementState::Released => {
                        self.state.input.mouse_buttons_down.remove(&mb);
                        MouseEvent::Released(mb)
                    }
                };

                self.app.on_mouse(&mut self.state, event);
            }

            WindowEvent::CursorMoved { position, .. } => {
                let screen_pos = Vec2::new(position.x as f32, position.y as f32);
                self.state.input.mouse_screen_pos = screen_pos;

                if let Some(renderer) = &self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    self.state.input.mouse_world_pos =
                        self.state.camera.screen_to_world(screen_pos, w, h);
                }

                let world_pos = self.state.input.mouse_world_pos;
                self.app.on_mouse(&mut self.state, MouseEvent::Moved(world_pos));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.state.input.scroll_delta += scroll;
                self.app.on_mouse(&mut self.state, MouseEvent::Scrolled(scroll));
            }

            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.state.renderer {
                    r.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                let frame_dt = if let Some(timer) = self.state.timer {
                    let dt = timer.elapsed().as_secs_f32();
                    self.state.timer = Some(Instant::now());
                    dt
                } else {
                    0.0
                };

                if let Some(start) = self.state.start {
                    self.state.elapsed = start.elapsed().as_secs_f32();
                }

                // Fixed timestep accumulation
                self.state.accumulator += frame_dt;
                let fixed_dt = self.state.fixed_dt;
                self.state.dt = fixed_dt;

                while self.state.accumulator >= fixed_dt {
                    self.state.input.begin_frame();
                    self.app.update(&mut self.state);
                    self.state.accumulator -= fixed_dt;
                }

                self.state.alpha = self.state.accumulator / fixed_dt;

                // Draw
                self.state.cells.clear();
                self.app.draw(&mut self.state);

                // Extract lights (max 64) — only cells with radius > 0 illuminate surroundings
                let mut lights: Vec<LightData> = Vec::new();
                for cell in &self.state.cells {
                    if cell.light_radius > 0.0 && lights.len() < 64 {
                        lights.push(cell.to_light_data());
                    }
                }

                // Extract bloom sources — all emissive/light cells get bloom
                let mut bloom_sources: Vec<LightData> = Vec::new();
                for cell in &self.state.cells {
                    if cell.light_radius >= 0.0 {
                        let mut ld = cell.to_light_data();
                        if ld.radius < 1.0 {
                            ld.radius = 1.0;
                        }
                        bloom_sources.push(ld);
                    }
                }

                // Partition into opaque and transparent, sort transparent by Z
                let mut opaque: Vec<CellInstance> = Vec::new();
                let mut transparent: Vec<CellInstance> = Vec::new();

                for cell in &self.state.cells {
                    let instance = cell.to_instance();
                    if cell.is_opaque() {
                        opaque.push(instance);
                    } else {
                        transparent.push(instance);
                    }
                }

                transparent.sort_by(|a, b| {
                    a.position[2].partial_cmp(&b.position[2]).unwrap_or(std::cmp::Ordering::Equal)
                });

                if let Some(renderer) = &mut self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    let (proj, offset, size) = self.state.camera.projection(w, h);

                    match renderer.render(
                        &opaque,
                        &transparent,
                        &lights,
                        &bloom_sources,
                        self.state.ambient_illumination,
                        proj,
                        offset,
                        size,
                        self.state.window_bg,
                        self.state.viewport_bg,
                    ) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let (w, h) = (renderer.config.width, renderer.config.height);
                            renderer.resize(w, h);
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Surface error: {e:?}"),
                    }
                }

                if let Some(w) = &self.state.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}
