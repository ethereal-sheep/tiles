use tiles::{
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, State,
};

struct LightingDemo {
    time: f32,
    ambient: f32,
    light_radius: f32,
    light_intensity: f32,
    num_orbiting: usize,
}

impl LightingDemo {
    fn new() -> Self {
        Self {
            time: 0.0,
            ambient: 0.05,
            light_radius: 20.0,
            light_intensity: 1.0,
            num_orbiting: 3,
        }
    }
}

impl App for LightingDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.02, 0.02, 0.03, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(self.ambient);
    }

    fn update(&mut self, state: &mut State) {
        self.time += state.dt();
        state.set_ambient_illumination(self.ambient);
    }

    fn draw(&mut self, state: &mut State) {
        let half = 32;

        // Draw a grid of cells as the "floor"
        for gy in -half..half {
            for gx in -half..half {
                let cx = gx as f32 + 0.5;
                let cy = gy as f32 + 0.5;
                let checker = ((gx + gy) & 1) == 0;
                let (r, g, b) = if checker {
                    (0.25, 0.22, 0.3)
                } else {
                    (0.18, 0.16, 0.22)
                };
                state.draw_world(Cell::new(cx, cy).color(Color::linear(r, g, b, 1.0)));
            }
        }

        // Orbiting lights
        for i in 0..self.num_orbiting {
            let angle =
                self.time * 0.8 + (i as f32) * std::f32::consts::TAU / self.num_orbiting as f32;
            let orbit_radius = 18.0;
            let x = angle.cos() * orbit_radius;
            let y = angle.sin() * orbit_radius;

            let hue = (i as f32) / self.num_orbiting as f32;
            let (r, g, b) = hue_to_rgb(hue);

            state.draw_world(
                Cell::new(x, y)
                    .color(Color::linear(r, g, b, 1.0))
                    .light(self.light_radius)
                    .intensity(self.light_intensity),
            );
        }

        // Mouse light (follows cursor)
        if state.is_mouse_down(MouseButton::Left) {
            let pos = state.mouse_world_position();
            state.draw_world(
                Cell::new(pos.x, pos.y)
                    .color(Color::linear(1.0, 0.95, 0.8, 1.0))
                    .light(self.light_radius * 1.5)
                    .intensity(1.5),
            );
        }

        // Center emissive cell
        let pulse = (self.time * 2.0).sin() * 0.3 + 0.7;
        state.draw_world(
            Cell::new(0.0, 0.0)
                .color(Color::linear(1.0, 0.4, 0.1, 1.0))
                .emissive()
                .intensity(pulse),
        );
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit(),
            KeyCode::Up => self.light_radius = (self.light_radius + 2.0).min(60.0),
            KeyCode::Down => self.light_radius = (self.light_radius - 2.0).max(2.0),
            KeyCode::Key1 => self.ambient = (self.ambient - 0.05).max(0.0),
            KeyCode::Key2 => self.ambient = (self.ambient + 0.05).min(1.0),
            KeyCode::Key3 => self.light_intensity = (self.light_intensity - 0.1).max(0.1),
            KeyCode::Key4 => self.light_intensity = (self.light_intensity + 0.1).min(3.0),
            KeyCode::Key5 => self.num_orbiting = (self.num_orbiting.saturating_sub(1)).max(1),
            KeyCode::Key6 => self.num_orbiting = (self.num_orbiting + 1).min(8),
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn hue_to_rgb(h: f32) -> (f32, f32, f32) {
    let h = h.fract();
    let h = if h < 0.0 { h + 1.0 } else { h };
    let sat = 0.8;
    let val = 1.0;
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = val * (1.0 - sat);
    let q = val * (1.0 - sat * f);
    let t = val * (1.0 - sat * (1.0 - f));
    match i % 6 {
        0 => (val, t, p),
        1 => (q, val, p),
        2 => (p, val, t),
        3 => (p, q, val),
        4 => (t, p, val),
        _ => (val, p, q),
    }
}

fn main() {
    let config = Config::builder()
        .title("Lighting Demo")
        .width(900)
        .height(900)
        .viewport(64.0, 64.0)
        .no_file()
        .build();

    tiles::run(LightingDemo::new(), config).unwrap();
}
