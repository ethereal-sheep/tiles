use glam::Vec2;
use tiles::{
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseAction, MouseButton, MouseEvent,
    State,
};

const G: f32 = 500.0;
const SOFTENING: f32 = 1.0;
const MERGE_BASE: f32 = 0.3;
const MERGE_K: f32 = 0.4;
const DEFAULT_COUNT: usize = 500;

struct Body {
    pos: Vec2,
    vel: Vec2,
    mass: f32,
}

struct NBodySim {
    bodies: Vec<Body>,
    time_scale: f32,
    time: f32,
    drag_start: Option<Vec2>,
    gravity_well: Option<Vec2>,
}

impl NBodySim {
    fn new() -> Self {
        let mut sim = Self {
            bodies: Vec::new(),
            time_scale: 1.0,
            time: 0.0,
            drag_start: None,
            gravity_well: None,
        };
        sim.reset_rotating_disk();
        sim
    }

    fn reset_random_disk(&mut self) {
        self.bodies.clear();
        for i in 0..DEFAULT_COUNT {
            let seed = i as f32 * 2.39996;
            let r = noise(seed + 1.0).abs() * 30.0;
            let angle = noise(seed + 2.0) * std::f32::consts::PI;
            let pos = Vec2::new(angle.cos() * r, angle.sin() * r);
            let vel = Vec2::new(noise(seed + 3.0) * 5.0, noise(seed + 4.0) * 5.0);
            self.bodies.push(Body {
                pos,
                vel,
                mass: 1.0,
            });
        }
    }

    fn reset_rotating_disk(&mut self) {
        self.bodies.clear();
        for i in 0..DEFAULT_COUNT {
            let seed = i as f32 * 2.39996;
            let r = 5.0 + noise(seed + 1.0).abs() * 30.0;
            let angle = noise(seed + 2.0) * std::f32::consts::PI;
            let pos = Vec2::new(angle.cos() * r, angle.sin() * r);
            let speed = (G * DEFAULT_COUNT as f32 / r).sqrt() * 0.3;
            let tangent = Vec2::new(-angle.sin(), angle.cos());
            let vel = tangent * speed + Vec2::new(noise(seed + 5.0) * 2.0, noise(seed + 6.0) * 2.0);
            self.bodies.push(Body {
                pos,
                vel,
                mass: 1.0,
            });
        }
    }

    fn reset_clusters(&mut self) {
        self.bodies.clear();
        let centers = [
            Vec2::new(-20.0, 15.0),
            Vec2::new(20.0, -10.0),
            Vec2::new(0.0, -20.0),
        ];
        let cluster_vel = [
            Vec2::new(8.0, -5.0),
            Vec2::new(-8.0, 3.0),
            Vec2::new(0.0, 7.0),
        ];
        let per_cluster = DEFAULT_COUNT / 3;
        for (ci, (center, bulk_vel)) in centers.iter().zip(cluster_vel.iter()).enumerate() {
            for i in 0..per_cluster {
                let seed = (ci * 1000 + i) as f32 * 2.39996;
                let r = noise(seed + 1.0).abs() * 10.0;
                let angle = noise(seed + 2.0) * std::f32::consts::PI;
                let pos = *center + Vec2::new(angle.cos() * r, angle.sin() * r);
                let vel = *bulk_vel + Vec2::new(noise(seed + 3.0) * 3.0, noise(seed + 4.0) * 3.0);
                self.bodies.push(Body {
                    pos,
                    vel,
                    mass: 1.0,
                });
            }
        }
    }

    fn merge_threshold(m1: f32, m2: f32) -> f32 {
        MERGE_BASE + MERGE_K * (m1 + m2).cbrt()
    }

    fn color_temperature(mass: f32) -> (f32, f32, f32) {
        let t = ((mass.ln() + 1.0) / 5.0).clamp(0.0, 1.0);
        if t < 0.3 {
            let u = t / 0.3;
            (0.6 + u * 0.4, 0.1 + u * 0.2, 0.02)
        } else if t < 0.7 {
            let u = (t - 0.3) / 0.4;
            (1.0, 0.3 + u * 0.5, 0.02 + u * 0.2)
        } else {
            let u = (t - 0.7) / 0.3;
            (1.0, 0.8 + u * 0.2, 0.22 + u * 0.78)
        }
    }

    fn light_radius(mass: f32) -> f32 {
        mass.sqrt() * 2.0
    }
}

fn noise(s: f32) -> f32 {
    (s.sin() * 43758.5453).fract() * 2.0 - 1.0
}

impl App for NBodySim {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.0, 0.0, 0.0, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(0.02);
    }

    fn update(&mut self, state: &mut State) {
        let dt = state.dt() * self.time_scale;
        self.time += dt;

        let n = self.bodies.len();
        if n == 0 {
            return;
        }

        // Gravity well from right mouse
        if let Some(well_pos) = self.gravity_well {
            let well_mass = 500.0;
            for body in &mut self.bodies {
                let diff = well_pos - body.pos;
                let dist_sq = diff.length_squared() + SOFTENING * SOFTENING;
                let force_mag = G * well_mass * body.mass / dist_sq;
                let acc = diff.normalize_or_zero() * force_mag / body.mass;
                body.vel += acc * dt;
            }
        }

        // Compute accelerations (O(n²))
        let mut acc = vec![Vec2::ZERO; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let diff = self.bodies[j].pos - self.bodies[i].pos;
                let dist_sq = diff.length_squared() + SOFTENING * SOFTENING;
                let force_mag = G * self.bodies[i].mass * self.bodies[j].mass / dist_sq;
                let dir = diff.normalize_or_zero();
                let force = dir * force_mag;
                acc[i] += force / self.bodies[i].mass;
                acc[j] -= force / self.bodies[j].mass;
            }
        }

        // Symplectic Euler: update velocity then position
        for (i, body) in self.bodies.iter_mut().enumerate() {
            body.vel += acc[i] * dt;
            body.pos += body.vel * dt;
        }

        // Merge pass
        let mut merged = vec![false; n];
        let mut new_bodies: Vec<Body> = Vec::new();

        for i in 0..n {
            if merged[i] {
                continue;
            }
            let mut body = Body {
                pos: self.bodies[i].pos,
                vel: self.bodies[i].vel,
                mass: self.bodies[i].mass,
            };
            for j in (i + 1)..n {
                if merged[j] {
                    continue;
                }
                let dist = (body.pos - self.bodies[j].pos).length();
                let threshold = NBodySim::merge_threshold(body.mass, self.bodies[j].mass);
                if dist < threshold {
                    let total_mass = body.mass + self.bodies[j].mass;
                    body.vel = (body.vel * body.mass + self.bodies[j].vel * self.bodies[j].mass)
                        / total_mass;
                    body.pos = (body.pos * body.mass + self.bodies[j].pos * self.bodies[j].mass)
                        / total_mass;
                    body.mass = total_mass;
                    merged[j] = true;
                }
            }
            new_bodies.push(body);
        }

        self.bodies = new_bodies;
    }

    fn draw(&mut self, state: &mut State) {
        for body in &self.bodies {
            let (r, g, b) = NBodySim::color_temperature(body.mass);
            let lr = NBodySim::light_radius(body.mass);

            let mut cell = Cell::new(body.pos.x, body.pos.y)
                .color(Color::linear(r, g, b, 1.0))
                .emissive();

            if lr > 0.5 {
                cell = Cell::new(body.pos.x, body.pos.y)
                    .color(Color::linear(r, g, b, 1.0))
                    .light(lr)
                    .intensity(body.mass.sqrt().min(3.0));
            }

            state.draw_world(cell);
        }

        // Draw drag line preview
        if let Some(start) = self.drag_start {
            let end = state.mouse_position();
            let dir = end - start;
            let len = dir.length();
            if len > 1.0 {
                let steps = (len as usize).min(20);
                for i in 0..steps {
                    let t = i as f32 / steps as f32;
                    let p = start + dir * t;
                    state.draw_world(
                        Cell::new(p.x, p.y)
                            .color(Color::linear(0.3, 0.3, 0.5, 0.5))
                            .emissive(),
                    );
                }
            }
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit(),
            KeyCode::Key1 => self.reset_random_disk(),
            KeyCode::Key2 => self.reset_rotating_disk(),
            KeyCode::Key3 => self.reset_clusters(),
            KeyCode::Up => self.time_scale = (self.time_scale * 1.5).min(10.0),
            KeyCode::Down => self.time_scale = (self.time_scale / 1.5).max(0.1),
            _ => {}
        }
    }

    fn on_mouse(&mut self, state: &mut State, event: MouseEvent) {
        match event.action {
            MouseAction::Pressed(MouseButton::Left) => {
                self.drag_start = Some(event.world_pos);
            }
            MouseAction::Released(MouseButton::Left) => {
                if let Some(start) = self.drag_start.take() {
                    let vel = (event.world_pos - start) * 2.0;
                    self.bodies.push(Body {
                        pos: start,
                        vel,
                        mass: 3.0,
                    });
                }
            }
            MouseAction::Pressed(MouseButton::Right) => {
                self.gravity_well = Some(event.world_pos);
            }
            MouseAction::Released(MouseButton::Right) => {
                self.gravity_well = None;
            }
            MouseAction::Moved { .. } => {
                if state.is_mouse_down(MouseButton::Right) {
                    self.gravity_well = Some(event.world_pos);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let config = Config::builder()
        .title("N-Body")
        .width(900)
        .height(900)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    tiles::run(NBodySim::new(), config).unwrap();
}
