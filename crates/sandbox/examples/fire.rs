use glam::Vec2;
use tiles::{
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, Rotation, State,
};

struct FireDemo {
    particles: Vec<FireParticle>,
    fire_pos: Vec2,
    spawn_rate: u32,
    wind: f32,
    turbulence: f32,
    intensity: f32,
    time: f32,
}

struct FireParticle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
    seed: f32,
}

impl FireParticle {
    fn life_ratio(&self) -> f32 {
        (self.life / self.max_life).clamp(0.0, 1.0)
    }

    fn color(&self) -> (f32, f32, f32, f32) {
        let t = self.life_ratio();
        if t > 0.7 {
            let u = (t - 0.7) / 0.3;
            let r = 1.0;
            let g = 0.85 + u * 0.15;
            let b = 0.4 + u * 0.4;
            (r, g, b, 1.0)
        } else if t > 0.3 {
            let u = (t - 0.3) / 0.4;
            let r = 0.6 + u * 0.4;
            let g = 0.1 + u * 0.75;
            let b = 0.02 + u * 0.38;
            (r, g, b, 1.0)
        } else {
            let u = t / 0.3;
            let r = 0.15 + u * 0.45;
            let g = 0.02 + u * 0.08;
            let b = 0.01 + u * 0.01;
            let a = u * 0.8;
            (r, g, b, a)
        }
    }

    fn is_body(&self) -> bool {
        let t = self.life_ratio();
        t > 0.3 && t <= 0.7
    }
}

impl FireDemo {
    fn new() -> Self {
        Self {
            particles: Vec::new(),
            fire_pos: Vec2::new(0.0, -20.0),
            spawn_rate: 20,
            wind: 0.0,
            turbulence: 1.0,
            intensity: 1.0,
            time: 0.0,
        }
    }

    fn spawn(&mut self, t: f32, count: u32) {
        for i in 0..count {
            let seed = t * 137.0 + i as f32 * 7.31;
            let ny = noise(seed + 3.7);

            let vx_noise = noise(seed + 9.1);

            let angle = noise(seed + 11.3) * std::f32::consts::PI;
            let radius = noise(seed + 13.7).abs() * 1.0;
            let spawn_x = self.fire_pos.x + angle.cos() * radius;
            let spawn_y = self.fire_pos.y + angle.sin() * radius;

            let vel = Vec2::new(
                vx_noise * 12.0 + self.wind,
                50.0 + ny.abs() * 10.0 * self.intensity,
            );

            let max_life = 0.4 + noise(seed + 5.1).abs() * 0.1 * self.intensity;

            self.particles.push(FireParticle {
                pos: Vec2::new(spawn_x, spawn_y),
                vel,
                life: max_life,
                max_life,
                seed,
            });
        }
    }
}

fn noise(s: f32) -> f32 {
    (s.sin() * 43758.5453).fract() * 2.0 - 1.0
}

impl App for FireDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.0, 0.0, 0.0, 1.0));
        state.set_window_background(Color::linear(0.0, 0.0, 0.0, 1.0));
        state.set_ambient_illumination(1.0);
    }

    fn update(&mut self, state: &mut State) {
        let dt = state.dt();
        self.time += dt;

        if state.is_mouse_down(MouseButton::Left) {
            self.fire_pos = state.mouse_world_position();
        }

        let rate = (self.spawn_rate as f32 * self.intensity) as u32;
        self.spawn(self.time, rate.max(1));

        let tip = Vec2::new(self.fire_pos.x, self.fire_pos.y + 20.0);

        for p in &mut self.particles {
            let turb_x = (self.time * 4.0 + p.seed).sin() * self.turbulence * 5.0;
            let turb_y = (self.time * 3.0 + p.seed * 1.3).cos() * self.turbulence * 1.5;

            // Pull toward tip horizontally (stronger as particle ages)
            let age = 1.0 - p.life_ratio();
            let attract_x = (tip.x - p.pos.x) * age * 100.0;

            p.vel.x += (turb_x + self.wind + attract_x - p.vel.x * 0.5) * dt;
            p.vel.y += (turb_y - p.vel.y * 0.1) * dt * 10.0;
            p.vel.y *= 1.0 - 0.3 * dt;

            p.pos += p.vel * dt;
            p.life -= dt;
        }

        self.particles.retain(|p| p.life > 0.0);
    }

    fn draw(&mut self, state: &mut State) {
        for p in self.particles.iter() {
            let (r, g, b, a) = p.color();

            let mut cell = Cell::new(p.pos.x, p.pos.y)
                .color(Color::linear(r, g, b, a))
                .emissive();

            if p.is_body() {
                let rot = (self.time * 3.0 + p.seed * 2.0).sin() * 0.3;
                cell = cell.rotation(Rotation::Z(rot));
            }

            state.draw_world(cell);
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit(),
            KeyCode::Up => self.wind += 2.0,
            KeyCode::Down => self.wind -= 2.0,
            KeyCode::Key1 => self.intensity = (self.intensity + 0.2).min(3.0),
            KeyCode::Key2 => self.intensity = (self.intensity - 0.2).max(0.2),
            KeyCode::Key3 => self.turbulence = (self.turbulence + 0.2).min(3.0),
            KeyCode::Key4 => self.turbulence = (self.turbulence - 0.2).max(0.0),
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let config = Config::builder()
        .title("Fire")
        .width(900)
        .height(900)
        .viewport(128, 128)
        .no_file()
        .build();

    tiles::run(FireDemo::new(), config).unwrap();
}
