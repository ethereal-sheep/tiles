use glam::Vec2;
use tiles::{
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, State,
};

struct Sandbox {
    particles: Vec<Particle>,
    gravity: f32,
    spawn_rate: u32,
    mode: Mode,
}

#[derive(Clone, Copy)]
enum Mode {
    Fountain,
    Rain,
    Explosion,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    hue: f32,
}

impl Particle {
    fn alpha(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }

    fn color(&self) -> [f32; 4] {
        let (r, g, b) = hue_to_rgb(self.hue);
        [r, g, b, self.alpha()]
    }
}

fn hue_to_rgb(h: f32) -> (f32, f32, f32) {
    let h = h.fract();
    let s = 1.0;
    let v = 1.0;
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}

impl App for Sandbox {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.02, 0.02, 0.04, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
    }

    fn pre_update(&mut self, _state: &mut State) {}

    fn update(&mut self, state: &mut State) {
        let dt = state.dt;
        let t = state.elapsed;

        // Spawn particles
        if state.is_mouse_down(MouseButton::Left) {
            let origin = state.mouse_position();
            for i in 0..self.spawn_rate {
                let p = self.spawn(origin, t, i);
                self.particles.push(p);
            }
        }

        // Auto-spawn in fountain mode
        if matches!(self.mode, Mode::Fountain) {
            let origin = Vec2::new(0.0, -50.0);
            for i in 0..3 {
                let p = self.spawn(origin, t, i);
                self.particles.push(p);
            }
        }

        // Physics
        for p in &mut self.particles {
            p.vel.y -= self.gravity * dt;
            p.pos += p.vel * dt;
            p.life -= dt;

            // Bounce off viewport edges
            if p.pos.x < -63.0 {
                p.pos.x = -63.0;
                p.vel.x = p.vel.x.abs() * 0.7;
            } else if p.pos.x > 63.0 {
                p.pos.x = 63.0;
                p.vel.x = -p.vel.x.abs() * 0.7;
            }
            if p.pos.y < -63.0 {
                p.pos.y = -63.0;
                p.vel.y = p.vel.y.abs() * 0.5;
            }
        }

        self.particles.retain(|p| p.life > 0.0);
    }

    fn draw(&mut self, state: &mut State) {
        for p in &self.particles {
            let [r, g, b, a] = p.color();
            state.draw(Cell::new(p.pos.x, p.pos.y).color(Color::linear(r, g, b, a)));
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit = true,
            KeyCode::C => self.particles.clear(),
            KeyCode::Key1 => self.mode = Mode::Fountain,
            KeyCode::Key2 => self.mode = Mode::Rain,
            KeyCode::Key3 => self.mode = Mode::Explosion,
            KeyCode::Up => self.gravity = (self.gravity + 10.0).min(200.0),
            KeyCode::Down => self.gravity = (self.gravity - 10.0).max(0.0),
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

impl Sandbox {
    fn spawn(&self, origin: Vec2, t: f32, i: u32) -> Particle {
        let seed = t * 100.0 + i as f32 * 7.3;
        let noise = |s: f32| (s.sin() * 43758.5453).fract() * 2.0 - 1.0;

        let (vel, max_life) = match self.mode {
            Mode::Fountain => {
                let spread = 15.0;
                let vel = Vec2::new(noise(seed) * spread, 40.0 + noise(seed + 1.0) * 10.0);
                (vel, 3.0 + noise(seed + 2.0).abs())
            }
            Mode::Rain => {
                let vel = Vec2::new(noise(seed) * 2.0, -20.0 - noise(seed + 1.0).abs() * 30.0);
                let pos_offset = Vec2::new(noise(seed + 3.0) * 60.0, 60.0);
                return Particle {
                    pos: origin + pos_offset,
                    vel,
                    life: 4.0,
                    max_life: 4.0,
                    hue: 0.55 + noise(seed + 4.0) * 0.05,
                };
            }
            Mode::Explosion => {
                let angle = noise(seed) * std::f32::consts::PI;
                let speed = 30.0 + noise(seed + 1.0).abs() * 50.0;
                let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);
                (vel, 1.5 + noise(seed + 2.0).abs())
            }
        };

        Particle {
            pos: origin,
            vel,
            life: max_life,
            max_life,
            hue: (t * 0.1 + i as f32 * 0.05).fract(),
        }
    }
}

fn main() {
    let config = Config::builder()
        .title("Tiles Sandbox")
        .width(900)
        .height(900)
        .viewport(128.0, 128.0)
        .no_file()
        .build();

    let app = Sandbox {
        particles: Vec::new(),
        gravity: 50.0,
        spawn_rate: 8,
        mode: Mode::Fountain,
    };

    tiles::run(app, config).unwrap();
}
