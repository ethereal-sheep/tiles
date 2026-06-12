use glam::Vec2;
use pollster::block_on;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::cell::{Cell, CellInstance, LightData};
use crate::config::Config;
use crate::drawable::Drawable;
use crate::input::{
    self, InputState, KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent,
};
use crate::renderer::Renderer;
use crate::{camera::Camera, input::ButtonState};

pub trait App {
    fn init(&mut self, _state: &mut State) {}
    fn pre_update(&mut self, _state: &mut State) {}
    fn update(&mut self, _state: &mut State) {}
    fn draw(&mut self, _state: &mut State) {}
    fn on_key(&mut self, _state: &mut State, _event: KeyEvent) {}
    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

pub struct State {
    window: Option<std::sync::Arc<Window>>,
    renderer: Option<Renderer>,
    cells: Vec<Cell>,
    screen_cells: Vec<Cell>,
    config: Config,
    camera: Camera,
    input: InputState,
    timer: Option<Instant>,
    start: Option<Instant>,
    accumulator: f32,
    fixed_dt: f32,
    debug: bool,
    window_bg: [f32; 4],
    viewport_bg: [f32; 4],
    ambient_illumination: f32,
    pub dt: f32,
    pub elapsed: f32,
    pub alpha: f32,
    pub quit: bool,
}

impl State {
    fn new(config: Config) -> Self {
        let fixed_dt = 1.0 / config.steps_per_second as f32;
        let camera = Camera::new(config.viewport_width, config.viewport_height);
        Self {
            window: None,
            renderer: None,
            cells: Vec::new(),
            screen_cells: Vec::new(),
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

    pub fn draw(&mut self, drawable: impl Drawable) {
        let half_w = self.camera.viewport_width / 2.0;
        let half_h = self.camera.viewport_height / 2.0;
        let min_x = self.camera.position.x - half_w - 1.0;
        let max_x = self.camera.position.x + half_w + 1.0;
        let min_y = self.camera.position.y - half_h - 1.0;
        let max_y = self.camera.position.y + half_h + 1.0;
        drawable.emit_cells(&mut |cell| {
            if cell.position.x >= min_x
                && cell.position.x <= max_x
                && cell.position.y >= min_y
                && cell.position.y <= max_y
            {
                self.cells.push(cell);
            }
        });
    }

    pub fn draw_screen(&mut self, drawable: impl Drawable) {
        let vw = self.camera.viewport_width;
        let vh = self.camera.viewport_height;
        drawable.emit_cells(&mut |cell| {
            if cell.position.x >= -1.0
                && cell.position.x <= vw
                && cell.position.y >= -1.0
                && cell.position.y <= vh
            {
                self.screen_cells.push(cell);
            }
        });
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

    // --- Coordinate conversion ---

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let half_w = self.camera.viewport_width / 2.0;
        let half_h = self.camera.viewport_height / 2.0;
        let x = world_pos.x - self.camera.position.x + half_w;
        let y = half_h - (world_pos.y - self.camera.position.y);
        Vec2::new(x, y)
    }

    pub fn pixel_to_viewport(&self, pixel_pos: Vec2) -> Vec2 {
        if let Some(renderer) = &self.renderer {
            let w = renderer.width();
            let h = renderer.height();
            let scale = self.camera.fit_scale(w, h);
            let viewport_screen_w = self.camera.viewport_width * scale;
            let viewport_screen_h = self.camera.viewport_height * scale;
            let offset_x = (w - viewport_screen_w) / 2.0;
            let offset_y = (h - viewport_screen_h) / 2.0;
            let vx = (pixel_pos.x - offset_x) / scale;
            let vy = (pixel_pos.y - offset_y) / scale;
            Vec2::new(vx, vy)
        } else {
            pixel_pos
        }
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
        self.input.is_key_down(key)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.input.is_key_pressed(key)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.input.is_key_released(key)
    }

    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        self.input.is_key_clicked(key)
    }

    pub fn is_key_double_clicked(&self, key: KeyCode) -> bool {
        self.input.is_key_double_clicked(key)
    }

    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.input.is_key_held(key)
    }

    pub fn is_key_released_after_hold(&self, key: KeyCode) -> bool {
        self.input.is_key_released_after_hold(key)
    }

    pub fn is_mouse_down(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_down(mouse)
    }

    pub fn is_mouse_pressed(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_pressed(mouse)
    }

    pub fn is_mouse_released(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_released(mouse)
    }

    pub fn is_mouse_clicked(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_clicked(mouse)
    }

    pub fn is_mouse_double_clicked(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_double_clicked(mouse)
    }

    pub fn is_mouse_held(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_held(mouse)
    }

    pub fn is_mouse_released_after_hold(&self, mouse: MouseButton) -> bool {
        self.input.is_mouse_released_after_hold(mouse)
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
    }

    pub fn debug_text(&mut self, _text: &str, _x: f32, _y: f32) {
        if !self.debug {
            return;
        }
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

pub(crate) fn run_app(
    app: &mut (impl App + 'static),
    config: Config,
) -> Result<(), winit::error::EventLoopError> {
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
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
                    KeyState::Pressed => {
                        self.state.input.keys_down.insert(key);
                        self.state
                            .input
                            .keys_states
                            .entry(key)
                            .or_insert(ButtonState::new())
                            .pressed_this_frame = true;
                    }
                    KeyState::Released => {
                        self.state.input.keys_down.remove(&key);
                        self.state
                            .input
                            .keys_states
                            .entry(key)
                            .or_insert(ButtonState::new())
                            .pressed_this_frame = false;
                    }
                }

                self.app.on_key(
                    &mut self.state,
                    KeyEvent {
                        key,
                        state: key_state,
                    },
                );
            }

            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                let mb = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    winit::event::MouseButton::Middle => MouseButton::Middle,
                    _ => return,
                };

                let action = match btn_state {
                    ElementState::Pressed => {
                        self.state.input.mouse_buttons_down.insert(mb);
                        let state = self
                            .state
                            .input
                            .mouse_buttons_states
                            .entry(mb)
                            .or_insert(ButtonState::new());
                        state.is_down = true;
                        state.pressed_this_frame = true;
                        MouseAction::Pressed(mb)
                    }
                    ElementState::Released => {
                        self.state.input.mouse_buttons_down.remove(&mb);
                        let state = self
                            .state
                            .input
                            .mouse_buttons_states
                            .entry(mb)
                            .or_insert(ButtonState::new());
                        state.is_down = false;
                        state.pressed_this_frame = false;
                        MouseAction::Released(mb)
                    }
                };

                let viewport_pos = self
                    .state
                    .pixel_to_viewport(self.state.input.mouse_screen_pos);
                let mouse_event = MouseEvent {
                    action,
                    screen_pos: self.state.input.mouse_screen_pos,
                    world_pos: self.state.input.mouse_world_pos,
                    viewport_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
            }

            WindowEvent::CursorMoved { position, .. } => {
                let screen_pos = Vec2::new(position.x as f32, position.y as f32);

                let prev_screen = self.state.input.mouse_screen_pos;
                let prev_world = self.state.input.mouse_world_pos;

                self.state.input.prev_mouse_screen_pos = prev_screen;
                self.state.input.prev_mouse_world_pos = prev_world;
                self.state.input.mouse_screen_pos = screen_pos;

                if let Some(renderer) = &self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    self.state.input.mouse_world_pos =
                        self.state.camera.screen_to_world(screen_pos, w, h);
                }

                let world_pos = self.state.input.mouse_world_pos;
                let screen_delta = screen_pos - prev_screen;
                let world_delta = world_pos - prev_world;

                let viewport_pos = self.state.pixel_to_viewport(screen_pos);
                let mouse_event = MouseEvent {
                    action: MouseAction::Moved {
                        screen_delta,
                        world_delta,
                    },
                    screen_pos,
                    world_pos,
                    viewport_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.state.input.scroll_delta += scroll;

                let viewport_pos = self
                    .state
                    .pixel_to_viewport(self.state.input.mouse_screen_pos);
                let mouse_event = MouseEvent {
                    action: MouseAction::Scrolled(scroll),
                    screen_pos: self.state.input.mouse_screen_pos,
                    world_pos: self.state.input.mouse_world_pos,
                    viewport_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
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

                self.state.input.update(self.state.dt, self.state.elapsed);

                // pre_update: once per frame, before simulation ticks
                self.app.pre_update(&mut self.state);

                while self.state.accumulator >= fixed_dt {
                    self.state.input.begin_frame();
                    self.app.update(&mut self.state);
                    self.state.accumulator -= fixed_dt;
                }

                self.state.alpha = self.state.accumulator / fixed_dt;

                // Draw
                self.state.cells.clear();
                self.state.screen_cells.clear();
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
                    a.position[2]
                        .partial_cmp(&b.position[2])
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                // Build screen-space instances (draw-order, unlit)
                let screen_instances: Vec<CellInstance> = self
                    .state
                    .screen_cells
                    .iter()
                    .map(|cell| cell.to_screen_instance())
                    .collect();

                if let Some(renderer) = &mut self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    let (proj, offset, size) = self.state.camera.projection(w, h);
                    let vp_cells = Vec2::new(
                        self.state.camera.viewport_width,
                        self.state.camera.viewport_height,
                    );

                    match renderer.render(
                        &opaque,
                        &transparent,
                        &screen_instances,
                        &lights,
                        &bloom_sources,
                        self.state.ambient_illumination,
                        proj,
                        offset,
                        size,
                        vp_cells,
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

                self.state.input.reset();

                if let Some(w) = &self.state.window {
                    w.request_redraw();
                }
            }

            _ => {}
        }
    }
}
