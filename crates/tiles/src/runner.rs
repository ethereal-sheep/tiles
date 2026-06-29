use glam::Vec2;
use pollster::block_on;
use std::time::{Duration, Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::camera::Camera;
use crate::cell::{Cell, CellInstance, LightData};
use crate::config::Config;
use crate::drawable::Drawable;
use crate::element::{self, HitState};
use crate::input::{
    self, ButtonState, InputState, KeyCode, KeyEvent, KeyState, MouseAction, MouseButton,
    MouseEvent,
};
use crate::renderer::Renderer;
use crate::shape::Shape;

pub trait App {
    fn init(&mut self, _state: &mut State) {}
    fn pre_update(&mut self, _state: &mut State) {}
    fn update(&mut self, _state: &mut State) {}
    fn draw(&mut self, _state: &mut State) {}
    fn on_key(&mut self, _state: &mut State, _event: KeyEvent) {}
    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}

    fn ui(&self, _state: &State) -> crate::ui::Node<Self>
    where
        Self: Sized,
    {
        crate::ui::col()
    }
}

pub struct State {
    window: Option<std::sync::Arc<Window>>,
    renderer: Option<Renderer>,
    cells: Vec<Cell>,
    screen_cells: Vec<Cell>,
    world_overlay_cells: Vec<Cell>,
    screen_overlay_cells: Vec<Cell>,
    config: Config,
    camera: Camera,
    input: InputState,
    window_bg: [f32; 4],
    viewport_bg: [f32; 4],
    ambient_illumination: f32,
    fixed_dt: Duration,
    accumulator: Duration,
    dt: Duration,
    elapsed: Duration,
    frame_timer: Instant,
    start_timer: Instant,
    quit: bool,
    debug: bool,
    pub rejected_cell_count: u32,
}

impl State {
    fn new(config: Config) -> Self {
        let fixed_dt = Duration::from_secs_f32(1.0 / config.steps_per_second as f32);
        let camera = Camera::new(config.viewport_width, config.viewport_height);
        Self {
            rejected_cell_count: 0,
            window: None,
            renderer: None,
            cells: Vec::new(),
            screen_cells: Vec::new(),
            world_overlay_cells: Vec::new(),
            screen_overlay_cells: Vec::new(),
            config,
            camera,
            input: InputState::new(),
            window_bg: [0.0, 0.0, 0.0, 1.0],
            viewport_bg: [0.08, 0.08, 0.10, 1.0],
            ambient_illumination: 1.0,
            fixed_dt,
            accumulator: Duration::from_secs(0),
            frame_timer: Instant::now(),
            start_timer: Instant::now(),
            dt: fixed_dt,
            elapsed: Duration::from_secs(0),
            quit: false,
            debug: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(viewport_w: u32, viewport_h: u32) -> Self {
        let mut config = Config::default();
        config.viewport_width = viewport_w as f32;
        config.viewport_height = viewport_h as f32;
        config.no_file = true;
        Self::new(config)
    }

    #[cfg(test)]
    pub(crate) fn set_input(&mut self, input: InputState) {
        self.input = input;
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

    pub fn pixel_to_screen(&self, pixel_pos: Vec2) -> Vec2 {
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

    pub fn test_shape_world(&self, shape: &impl Shape) -> HitState {
        element::test_shape(&self.input, shape, false)
    }

    pub fn test_shape_screen(&self, shape: &impl Shape) -> HitState {
        element::test_shape(&self.input, shape, true)
    }

    // --- Drawing ---

    pub fn draw_world(&mut self, drawable: impl Drawable) {
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
            } else {
                self.rejected_cell_count += 1;
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
            } else {
                self.rejected_cell_count += 1;
            }
        });
    }

    pub fn draw_world_overlay(&mut self, drawable: impl Drawable) {
        let min_x = self.camera.position.x - self.camera.viewport_width / 2.0 - 1.0;
        let max_x = self.camera.position.x + self.camera.viewport_width / 2.0 + 1.0;
        let min_y = self.camera.position.y - self.camera.viewport_height / 2.0 - 1.0;
        let max_y = self.camera.position.y + self.camera.viewport_height / 2.0 + 1.0;
        drawable.emit_cells(&mut |cell| {
            if cell.position.x >= min_x
                && cell.position.x <= max_x
                && cell.position.y >= min_y
                && cell.position.y <= max_y
            {
                self.world_overlay_cells.push(cell);
            } else {
                self.rejected_cell_count += 1;
            }
        });
    }

    pub fn draw_screen_overlay(&mut self, drawable: impl Drawable) {
        let vw = self.camera.viewport_width;
        let vh = self.camera.viewport_height;
        drawable.emit_cells(&mut |cell| {
            if cell.position.x >= -1.0
                && cell.position.x <= vw
                && cell.position.y >= -1.0
                && cell.position.y <= vh
            {
                self.screen_overlay_cells.push(cell);
            } else {
                self.rejected_cell_count += 1;
            }
        });
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

    // --- Actions ---

    pub fn quit(&mut self) {
        self.quit = true;
    }

    pub fn dt(&self) -> f32 {
        self.dt.as_secs_f32()
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed.as_secs_f32()
    }
}

pub(crate) fn run_app<A: App + 'static>(
    app: &mut A,
    config: Config,
) -> Result<(), winit::error::EventLoopError> {
    let state = State::new(config);
    let event_loop = EventLoop::new().unwrap();

    let app_ptr = app as *mut A;
    let mut runner: Runner<A> = Runner {
        app: unsafe { &mut *app_ptr },
        state,
    };
    event_loop.run_app(&mut runner)
}

struct Runner<'a, A: App> {
    app: &'a mut A,
    state: State,
}

impl<A: App> ApplicationHandler for Runner<'_, A> {
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
        self.state.start_timer = Instant::now();
        self.state.frame_timer = Instant::now();
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
                        self.state
                            .input
                            .keys_states
                            .entry(key)
                            .or_insert(ButtonState::new())
                            .pressed_this_frame = true;
                    }
                    KeyState::Released => {
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
                        let state = self
                            .state
                            .input
                            .mouse_buttons_states
                            .entry(mb)
                            .or_insert(ButtonState::new());
                        state.is_down = true;
                        state.pressed_this_frame = true;
                        if mb == MouseButton::Left {
                            self.state.input.left_press_screen_pos =
                                self.state.input.mouse_screen_pos;
                            self.state.input.left_press_world_pos =
                                self.state.input.mouse_world_pos;
                        }
                        MouseAction::Pressed(mb)
                    }
                    ElementState::Released => {
                        let state = self
                            .state
                            .input
                            .mouse_buttons_states
                            .entry(mb)
                            .or_insert(ButtonState::new());
                        state.is_down = false;
                        state.released_this_frame = true;
                        MouseAction::Released(mb)
                    }
                };

                let screen_pos = self.state.input.mouse_screen_pos;
                let mouse_event = MouseEvent {
                    action,
                    screen_pos,
                    world_pos: self.state.input.mouse_world_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pixel_pos = Vec2::new(position.x as f32, position.y as f32);
                let screen_pos = self.state.pixel_to_screen(pixel_pos);

                let prev_screen = self.state.input.mouse_screen_pos;
                let prev_world = self.state.input.mouse_world_pos;

                self.state.input.prev_mouse_screen_pos = prev_screen;
                self.state.input.prev_mouse_world_pos = prev_world;
                self.state.input.mouse_screen_pos = screen_pos;

                if let Some(renderer) = &self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    self.state.input.mouse_world_pos =
                        self.state.camera.screen_to_world(pixel_pos, w, h);
                }

                let world_pos = self.state.input.mouse_world_pos;
                let screen_delta = screen_pos - prev_screen;
                let world_delta = world_pos - prev_world;

                let mouse_event = MouseEvent {
                    action: MouseAction::Moved {
                        screen_delta,
                        world_delta,
                    },
                    screen_pos,
                    world_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.state.input.scroll_delta += scroll;

                let screen_pos = self.state.input.mouse_screen_pos;
                let mouse_event = MouseEvent {
                    action: MouseAction::Scrolled(scroll),
                    screen_pos,
                    world_pos: self.state.input.mouse_world_pos,
                };

                self.app.on_mouse(&mut self.state, mouse_event);
            }

            WindowEvent::Resized(size) => {
                if let Some(r) = &mut self.state.renderer {
                    r.resize(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => {
                let frame_start = Instant::now();
                let frame_dt = frame_start - self.state.frame_timer;
                let consumed_tick_timer = self.state.frame_timer - self.state.accumulator;
                self.state.frame_timer = frame_start;
                self.state.accumulator += frame_dt;

                let mut update_count = 0;
                while self.state.accumulator >= self.state.fixed_dt {
                    self.state.accumulator -= self.state.fixed_dt;
                    update_count += 1;
                }

                let expanded_dt = self.state.fixed_dt * update_count;
                let elapsed_last = consumed_tick_timer - self.state.start_timer;

                if update_count == 0 {
                    if let Some(w) = &self.state.window {
                        w.request_redraw();
                    }
                    return;
                }

                self.state.input.update(
                    expanded_dt.as_secs_f32(),
                    (elapsed_last + expanded_dt).as_secs_f32(),
                );

                // Clear overlay buffers before pre_update populates them
                self.state.world_overlay_cells.clear();
                self.state.screen_overlay_cells.clear();

                // pre_update: once per frame, before simulation ticks
                self.state.dt = expanded_dt;
                self.state.elapsed = elapsed_last + expanded_dt;
                self.app.pre_update(&mut self.state);

                let tree = self.app.ui(&self.state);
                self.state.rejected_cell_count = 0;
                let resolved = tree.layout(
                    self.state.camera.viewport_width as u32,
                    self.state.camera.viewport_height as u32,
                );
                let (cells, _result) = resolved.evaluate(self.app, &mut self.state);
                self.state.draw_screen_overlay(cells);

                self.state.dt = self.state.fixed_dt;
                for i in 1..=update_count {
                    // Fixed timestep accumulation
                    self.state.input.begin_state_update();
                    self.state.elapsed = elapsed_last + i * self.state.fixed_dt;
                    self.app.update(&mut self.state);
                }

                // Draw
                self.state.dt = expanded_dt;
                self.state.elapsed = elapsed_last + expanded_dt;
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

                // World overlay renders after all world cells
                for cell in &self.state.world_overlay_cells {
                    transparent.push(cell.to_instance());
                }

                // Build screen-space instances (draw-order, unlit)
                let mut screen_instances: Vec<CellInstance> = self
                    .state
                    .screen_cells
                    .iter()
                    .map(|cell| cell.to_screen_instance())
                    .collect();

                // Screen overlay renders after all screen cells
                for cell in &self.state.screen_overlay_cells {
                    screen_instances.push(cell.to_screen_instance());
                }

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
