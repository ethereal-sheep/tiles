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
use crate::color::Color;
use crate::config::Config;
use crate::drawable::Drawable;
use crate::element::{self, HitState};
use crate::image::{Image, ImageError};
use crate::input::{
    self, ButtonState, InputState, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent,
};
use crate::rect::Rect;
use crate::renderer::{DebugVertex, Renderer};
use crate::shape::Shape;

pub trait App {
    fn init(&mut self, _state: &mut State) {}
    fn pre_update(&mut self, _state: &mut State) {}
    fn update(&mut self, _state: &mut State) {}
    fn draw(&mut self, _state: &mut State) {}
    fn on_key(&mut self, _state: &mut State, _event: KeyEvent) {}
    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}

    fn ui(&self, _state: &State) -> crate::node::Node<Self>
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
    debug_vertices: Vec<DebugVertex>,
    config: Config,
    camera: Camera,
    input: InputState,
    window_bg: Color,
    viewport_bg: Color,
    ambient_illumination: f32,
    fixed_dt: Duration,
    accumulator: Duration,
    dt: Duration,
    elapsed: Duration,
    frame_timer: Instant,
    start_timer: Instant,
    quit: bool,
    debug: bool,
    images: std::collections::HashMap<String, Image>,
}

impl State {
    fn new(config: Config) -> Self {
        let fixed_dt = Duration::from_secs_f32(1.0 / config.steps_per_second as f32);
        let camera = Camera::new(config.viewport_width, config.viewport_height);
        Self {
            window: None,
            renderer: None,
            cells: Vec::new(),
            screen_cells: Vec::new(),
            world_overlay_cells: Vec::new(),
            screen_overlay_cells: Vec::new(),
            debug_vertices: Vec::new(),
            config,
            camera,
            input: InputState::new(),
            window_bg: Color::hex(0x000000),
            viewport_bg: Color::hex(0x030304),
            ambient_illumination: 1.0,
            fixed_dt,
            accumulator: Duration::from_secs(0),
            frame_timer: Instant::now(),
            start_timer: Instant::now(),
            dt: fixed_dt,
            elapsed: Duration::from_secs(0),
            quit: false,
            debug: false,
            images: std::collections::HashMap::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(viewport_w: u32, viewport_h: u32) -> Self {
        let mut config = Config::default();
        config.viewport_width = viewport_w;
        config.viewport_height = viewport_h;
        config.no_file = true;
        Self::new(config)
    }

    #[cfg(test)]
    pub(crate) fn set_input(&mut self, input: InputState) {
        self.input = input;
    }

    #[cfg(test)]
    pub(crate) fn get_cells(&self) -> Vec<Cell> {
        [
            self.cells.as_slice(),
            self.world_overlay_cells.as_slice(),
            self.screen_cells.as_slice(),
            self.screen_overlay_cells.as_slice(),
        ]
        .concat()
    }

    pub(crate) fn get_input_ref(&self) -> &InputState {
        &self.input
    }

    pub(crate) fn get_input_mut_ref(&mut self) -> &mut InputState {
        &mut self.input
    }

    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.camera.set_viewport_size(width, height);
        self.config.viewport_width = width;
        self.config.viewport_height = height;
        self.config.save();
    }

    pub fn viewport_width(&self) -> u32 {
        self.camera.viewport_width()
    }

    pub fn viewport_height(&self) -> u32 {
        self.camera.viewport_height()
    }

    pub fn camera_position(&self) -> (f32, f32) {
        self.camera.position()
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32) {
        self.camera.set_position(x, y);
    }

    pub fn move_camera(&mut self, dx: f32, dy: f32) {
        let (x, y) = self.camera.position();
        self.camera.set_position(x + dx, y + dy);
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

    pub fn set_window_background(&mut self, color: Color) {
        self.window_bg = color;
    }

    pub fn set_viewport_background(&mut self, color: Color) {
        self.viewport_bg = color;
    }

    // --- Resources ---

    pub fn load_image(
        &mut self,
        key: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), ImageError> {
        let image = Image::from_path(path)?;
        self.images.insert(key.into(), image);
        Ok(())
    }

    pub fn image(&self, key: impl Into<String>) -> Option<&Image> {
        self.images.get(&key.into())
    }

    pub fn set_ambient_illumination(&mut self, ambient: f32) {
        self.ambient_illumination = ambient.clamp(0.0, 1.0);
    }

    pub fn test_shape_world(&self, shape: &impl Shape) -> HitState {
        element::test_shape(&self.input, shape, false)
    }

    pub fn test_shape_screen(&self, shape: &impl Shape) -> HitState {
        element::test_shape(&self.input, shape, true)
    }

    pub fn pixel_to_screen(&self, pixel_x: f32, pixel_y: f32) -> (f32, f32) {
        if let Some(renderer) = &self.renderer {
            self.camera
                .pixel_to_screen(renderer.width(), renderer.height(), pixel_x, pixel_y)
        } else {
            (pixel_x, pixel_y)
        }
    }

    pub fn screen_to_pixel(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        if let Some(renderer) = &self.renderer {
            self.camera
                .screen_to_pixel(renderer.width(), renderer.height(), screen_x, screen_y)
        } else {
            (screen_x, screen_y)
        }
    }

    pub fn pixel_to_world(&self, pixel_x: f32, pixel_y: f32) -> (f32, f32) {
        if let Some(renderer) = &self.renderer {
            self.camera
                .pixel_to_world(renderer.width(), renderer.height(), pixel_x, pixel_y)
        } else {
            (pixel_x, pixel_y)
        }
    }

    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        self.camera.world_to_screen(world_x, world_y)
    }

    pub fn screen_to_world(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        self.camera.screen_to_world(world_x, world_y)
    }

    fn is_rect_in_view(&self, rect: Rect, is_screen_space: bool) -> bool {
        if is_screen_space {
            self.camera.screen_space_manifold()
        } else {
            self.camera.world_space_manifold()
        }
        .expand_bottom(rect.height() as i32)
        .expand_top(rect.height() as i32)
        .expand_left(rect.width() as i32)
        .expand_right(rect.width() as i32)
        .contains_point(rect.x(), rect.y())
    }

    // --- Drawing ---
    fn draw(&mut self, drawable: impl Drawable, is_screen_space: bool, is_overlay: bool) {
        if is_screen_space {
            drawable.emit_cells(&mut |cell| {
                if self.is_rect_in_view(cell.to_rect(), is_screen_space) {
                    if is_overlay {
                        &mut self.screen_overlay_cells
                    } else {
                        &mut self.screen_cells
                    }
                    .push(cell);
                }
            });
        } else {
            drawable.flip_y().emit_cells(&mut |cell| {
                if self.is_rect_in_view(cell.to_rect(), is_screen_space) {
                    if is_overlay {
                        &mut self.world_overlay_cells
                    } else {
                        &mut self.cells
                    }
                    .push(cell);
                }
            });
        }
    }

    pub fn draw_world(&mut self, drawable: impl Drawable) {
        self.draw(drawable, false, false);
    }

    pub fn draw_screen(&mut self, drawable: impl Drawable) {
        self.draw(drawable, true, false);
    }

    pub(crate) fn draw_world_overlay(&mut self, drawable: impl Drawable) {
        self.draw(drawable, false, true);
    }

    pub(crate) fn draw_screen_overlay(&mut self, drawable: impl Drawable) {
        self.draw(drawable, true, true);
    }

    // --- Debug ---

    pub fn set_debug(&mut self, enabled: bool) {
        self.debug = enabled;
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }

    pub fn debug_line(&mut self, from: Vec2, to: Vec2, color: Color) {
        if !self.debug {
            return;
        }
        let (sx, sy) = self.world_to_screen(from.x, from.y);
        let from_px = self.screen_to_pixel(sx, sy);
        let (sx, sy) = self.world_to_screen(to.x, to.y);
        let to_px = self.screen_to_pixel(sx, sy);

        self.push_debug_line(from_px.0, from_px.1, to_px.0, to_px.1, color);
    }

    pub fn debug_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        if !self.debug {
            return;
        }
        let tl = Vec2::new(x, y);
        let tr = Vec2::new(x + w, y);
        let br = Vec2::new(x + w, y + h);
        let bl = Vec2::new(x, y + h);
        self.debug_line(tl, tr, color);
        self.debug_line(tr, br, color);
        self.debug_line(br, bl, color);
        self.debug_line(bl, tl, color);
    }

    pub fn debug_text(&mut self, _text: &str, _x: f32, _y: f32) {
        if !self.debug {
            return;
        }
    }

    fn push_debug_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        let c = color.to_array();
        let dx = x1 - x0;
        let dy = y1 - y0;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-6 {
            return;
        }
        let nx = -dy / len * 0.5;
        let ny = dx / len * 0.5;
        let v0 = [x0 + nx, y0 + ny];
        let v1 = [x0 - nx, y0 - ny];
        let v2 = [x1 + nx, y1 + ny];
        let v3 = [x1 - nx, y1 - ny];
        self.debug_vertices.push(DebugVertex {
            position: v0,
            color: c,
        });
        self.debug_vertices.push(DebugVertex {
            position: v1,
            color: c,
        });
        self.debug_vertices.push(DebugVertex {
            position: v2,
            color: c,
        });
        self.debug_vertices.push(DebugVertex {
            position: v2,
            color: c,
        });
        self.debug_vertices.push(DebugVertex {
            position: v1,
            color: c,
        });
        self.debug_vertices.push(DebugVertex {
            position: v3,
            color: c,
        });
    }

    pub(crate) fn debug_ui_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        if w < 1.0 || h < 1.0 {
            return;
        }

        let (px, py) = self.screen_to_pixel(x, y);
        let (px2, py2) = self.screen_to_pixel(x + w, y + h);

        self.push_debug_line(px, py, px2, py, color);
        self.push_debug_line(px2, py, px2, py2, color);
        self.push_debug_line(px2, py2, px, py2, color);
        self.push_debug_line(px, py2, px, py, color);
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
        signal_runtime: crate::signal::SignalRuntime::new(),
    };
    event_loop.run_app(&mut runner)
}

struct Runner<'a, A: App> {
    app: &'a mut A,
    state: State,
    signal_runtime: crate::signal::SignalRuntime,
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
                let prev_screen = self.state.input.mouse_screen_pos;
                let prev_world = self.state.input.mouse_world_pos;

                self.state.input.mouse_screen_pos = Vec2::from(
                    self.state
                        .pixel_to_screen(position.x as f32, position.y as f32),
                );

                self.state.input.mouse_world_pos = self
                    .state
                    .pixel_to_world(position.x as f32, position.y as f32)
                    .into();

                let screen_delta = self.state.input.mouse_screen_pos - prev_screen;
                let world_delta = self.state.input.mouse_world_pos - prev_world;

                let mouse_event = MouseEvent {
                    action: MouseAction::Moved {
                        screen_delta,
                        world_delta,
                    },
                    screen_pos: self.state.input.mouse_screen_pos,
                    world_pos: self.state.input.mouse_world_pos,
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

                self.state.debug_vertices.clear();

                crate::signal::set_runtime(&self.signal_runtime);
                let tree = self.app.ui(&self.state);
                let resolved = tree.layout(
                    self.state.viewport_width(),
                    self.state.viewport_height(),
                    &self.state,
                );

                if self.state.is_debug() {
                    dbg!(&resolved);
                }
                resolved.evaluate(self.app, &mut self.state);
                crate::signal::clear_runtime();

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
                let viewport_cells = Vec2::new(
                    self.state.viewport_width() as f32,
                    self.state.viewport_height() as f32,
                );
                if let Some(renderer) = &mut self.state.renderer {
                    let w = renderer.width();
                    let h = renderer.height();
                    let (projection, offset, size) = self.state.camera.projection(w, h);

                    match renderer.render(
                        &opaque,
                        &transparent,
                        &screen_instances,
                        &self.state.debug_vertices,
                        &lights,
                        &bloom_sources,
                        self.state.ambient_illumination,
                        projection,
                        offset,
                        size,
                        viewport_cells,
                        self.state.window_bg.to_array(),
                        self.state.viewport_bg.to_array(),
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
