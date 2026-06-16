use glam::Vec2;
use tiles::{
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, Rotation, State,
};

const FRAC_PI_2: f32 = std::f32::consts::FRAC_PI_2;

struct Boids {
    flock: Vec<Boid>,

    // Tunables
    separation_radius: f32,
    alignment_radius: f32,
    cohesion_radius: f32,
    separation_strength: f32,
    alignment_strength: f32,
    cohesion_strength: f32,
    max_speed: f32,
    min_speed: f32,
    edge_margin: f32,
    edge_turn_force: f32,
}

struct Boid {
    pos: Vec2,
    vel: Vec2,
    hue: f32,
}

impl Boids {
    fn new(count: usize, viewport: f32) -> Self {
        let half = viewport / 2.0;
        let mut flock = Vec::with_capacity(count);

        for i in 0..count {
            let seed = i as f32;
            let x = hash(seed * 1.1) * viewport - half;
            let y = hash(seed * 2.3) * viewport - half;
            let angle = hash(seed * 3.7) * std::f32::consts::TAU;
            let speed = 15.0 + hash(seed * 4.9) * 10.0;
            flock.push(Boid {
                pos: Vec2::new(x, y),
                vel: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                hue: hash(seed * 5.1),
            });
        }

        Self {
            flock,
            separation_radius: 5.0,
            alignment_radius: 12.0,
            cohesion_radius: 14.0,
            separation_strength: 60.0,
            alignment_strength: 2.0,
            cohesion_strength: 0.5,
            max_speed: 30.0,
            min_speed: 10.0,
            edge_margin: 15.0,
            edge_turn_force: 50.0,
        }
    }
}

fn hash(x: f32) -> f32 {
    (x.sin() * 43758.5453).fract().abs()
}

impl App for Boids {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.03, 0.03, 0.05, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
    }

    fn update(&mut self, state: &mut State) {
        let dt = state.dt();
        let half = 64.0;

        // Collect positions/velocities for neighbor queries
        let positions: Vec<Vec2> = self.flock.iter().map(|b| b.pos).collect();
        let velocities: Vec<Vec2> = self.flock.iter().map(|b| b.vel).collect();

        for i in 0..self.flock.len() {
            let mut separation = Vec2::ZERO;
            let mut alignment = Vec2::ZERO;
            let mut cohesion = Vec2::ZERO;
            let mut align_count = 0u32;
            let mut cohesion_count = 0u32;

            for j in 0..self.flock.len() {
                if i == j {
                    continue;
                }
                let diff = positions[i] - positions[j];
                let dist = diff.length();

                if dist < self.separation_radius && dist > 0.0 {
                    separation += diff / (dist * dist);
                }
                if dist < self.alignment_radius {
                    alignment += velocities[j];
                    align_count += 1;
                }
                if dist < self.cohesion_radius {
                    cohesion += positions[j];
                    cohesion_count += 1;
                }
            }

            let mut accel = Vec2::ZERO;

            // Separation
            accel += separation * self.separation_strength;

            // Alignment
            if align_count > 0 {
                let avg_vel = alignment / align_count as f32;
                accel += (avg_vel - self.flock[i].vel) * self.alignment_strength;
            }

            // Cohesion
            if cohesion_count > 0 {
                let center = cohesion / cohesion_count as f32;
                accel += (center - self.flock[i].pos) * self.cohesion_strength;
            }

            // Edge avoidance
            let pos = self.flock[i].pos;
            if pos.x < -half + self.edge_margin {
                accel.x += self.edge_turn_force;
            }
            if pos.x > half - self.edge_margin {
                accel.x -= self.edge_turn_force;
            }
            if pos.y < -half + self.edge_margin {
                accel.y += self.edge_turn_force;
            }
            if pos.y > half - self.edge_margin {
                accel.y -= self.edge_turn_force;
            }

            // Mouse attraction
            if state.is_mouse_down(MouseButton::Left) {
                let target = state.mouse_position();
                let to_target = target - self.flock[i].pos;
                accel += to_target * 3.0;
            }
            // Mouse repulsion
            if state.is_mouse_down(MouseButton::Right) {
                let target = state.mouse_position();
                let away = self.flock[i].pos - target;
                let dist = away.length().max(1.0);
                accel += away / dist * 80.0;
            }

            self.flock[i].vel += accel * dt;

            // Clamp speed
            let speed = self.flock[i].vel.length();
            if speed > self.max_speed {
                self.flock[i].vel = self.flock[i].vel / speed * self.max_speed;
            } else if speed < self.min_speed && speed > 0.0 {
                self.flock[i].vel = self.flock[i].vel / speed * self.min_speed;
            }

            let vel = self.flock[i].vel;
            self.flock[i].pos += vel * dt;
        }
    }

    fn draw(&mut self, state: &mut State) {
        for boid in &self.flock {
            let angle = boid.vel.y.atan2(boid.vel.x);
            // Rotation::Z maps 0→1 to 0→90°, so divide by PI/2
            let rot_val = angle / FRAC_PI_2;
            let (r, g, b) = hue_to_rgb(boid.hue);
            state.draw(
                Cell::new(boid.pos.x, boid.pos.y)
                    .rotation(Rotation::Z(rot_val))
                    .color(Color::linear(r, g, b, 1.0)),
            );
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit(),
            KeyCode::Up => self.max_speed = (self.max_speed + 5.0).min(80.0),
            KeyCode::Down => self.max_speed = (self.max_speed - 5.0).max(5.0),
            KeyCode::Key1 => self.separation_strength = (self.separation_strength + 5.0).min(100.0),
            KeyCode::Key2 => self.separation_strength = (self.separation_strength - 5.0).max(0.0),
            KeyCode::Key3 => self.alignment_strength = (self.alignment_strength + 1.0).min(20.0),
            KeyCode::Key4 => self.alignment_strength = (self.alignment_strength - 1.0).max(0.0),
            KeyCode::Key5 => self.cohesion_strength = (self.cohesion_strength + 1.0).min(20.0),
            KeyCode::Key6 => self.cohesion_strength = (self.cohesion_strength - 1.0).max(0.0),
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn hue_to_rgb(h: f32) -> (f32, f32, f32) {
    let h = h.fract();
    let h = if h < 0.0 { h + 1.0 } else { h };
    let sat = 0.6;
    let val = 0.9;
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
        .title("Boids")
        .width(900)
        .height(900)
        .viewport(128.0, 128.0)
        .no_file()
        .build();

    tiles::run(Boids::new(300, 128.0), config).unwrap();
}
