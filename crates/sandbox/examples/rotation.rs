use tiles::{App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseEvent, Rotation, State};

struct RotationDemo {
    grid_size: i32,
    cells_hue: Vec<f32>,
    current_hue: f32,
    hue_step: f32,

    // Wave state
    wave_active: bool,
    wave_time: f32,
    wave_origin: (f32, f32),
    wave_rotation: WaveRotation,
    target_hue: f32,

    // Tunables
    flip_duration: f32,
    delay_per_unit: f32,
    pause_duration: f32,
    overshoot: f32,
    pause_timer: f32,
    paused: bool,
}

#[derive(Clone, Copy)]
enum WaveRotation {
    FlipX,
    FlipY,
    DiagonalTL,
    DiagonalTR,
    Z,
}

impl RotationDemo {
    fn new() -> Self {
        let grid_size = 64;
        let total = (grid_size * grid_size) as usize;
        let initial_hue = 0.0;

        Self {
            grid_size,
            cells_hue: vec![initial_hue; total],
            current_hue: initial_hue,
            hue_step: 0.15,

            wave_active: false,
            wave_time: 0.0,
            wave_origin: (0.0, 0.0),
            wave_rotation: WaveRotation::FlipY,
            target_hue: 0.0,

            flip_duration: 0.6,
            delay_per_unit: 0.015,
            pause_duration: 0.5,
            overshoot: 0.00,
            pause_timer: 0.0,
            paused: false,
        }
    }

    fn trigger_wave(&mut self, time_seed: f32) {
        let pick = ((time_seed * 73.13).sin().abs() * 8.0) as u32 % 8;

        let half = self.grid_size as f32 / 2.0;
        let (origin, rotation) = match pick {
            0 => ((-half, 0.0), WaveRotation::FlipY),       // left edge
            1 => ((half, 0.0), WaveRotation::FlipY),        // right edge
            2 => ((0.0, half), WaveRotation::FlipX),        // top edge
            3 => ((0.0, -half), WaveRotation::FlipX),       // bottom edge
            4 => ((-half, half), WaveRotation::DiagonalTR), // top-left corner
            5 => ((half, half), WaveRotation::DiagonalTL),  // top-right corner
            6 => ((-half, -half), WaveRotation::DiagonalTL),// bottom-left corner
            7 => ((half, -half), WaveRotation::DiagonalTR), // bottom-right corner
            _ => ((0.0, 0.0), WaveRotation::Z),
        };

        self.wave_origin = origin;
        self.wave_rotation = rotation;
        self.wave_active = true;
        self.wave_time = 0.0;
        self.target_hue = self.current_hue + self.hue_step;
    }

    fn wave_max_distance(&self) -> f32 {
        let half = self.grid_size as f32 / 2.0;
        let dx = self.wave_origin.0.abs() + half;
        let dy = self.wave_origin.1.abs() + half;
        (dx * dx + dy * dy).sqrt()
    }

    fn wave_total_duration(&self) -> f32 {
        self.wave_max_distance() * self.delay_per_unit + self.flip_duration
    }

    fn flip_progress(&self, cell_x: f32, cell_y: f32) -> f32 {
        let dx = cell_x - self.wave_origin.0;
        let dy = cell_y - self.wave_origin.1;
        let dist = (dx * dx + dy * dy).sqrt();
        let start_time = dist * self.delay_per_unit;
        let local_t = (self.wave_time - start_time) / self.flip_duration;
        local_t
    }

    fn bounce_ease(t: f32, _overshoot: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        t
        // if t < 0.8 {
        //     // Smooth rise to 1.0 + overshoot
        //     let u = t / 0.8;
        //     let smooth = u * u * (3.0 - 2.0 * u);
        //     smooth * (1.0 + overshoot)
        // } else {
        //     // Settle from 1.0 + overshoot back to 1.0
        //     let u = (t - 0.8) / 0.2;
        //     (1.0 + overshoot) - overshoot * u
        // }
    }

    fn cell_index(&self, gx: i32, gy: i32) -> usize {
        (gy * self.grid_size + gx) as usize
    }
}

fn hue_to_rgb(h: f32) -> (f32, f32, f32) {
    let h = h.fract();
    let h = if h < 0.0 { h + 1.0 } else { h };
    let sat = 0.45;
    let val = 0.85;
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

impl App for RotationDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.02, 0.02, 0.03, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        self.pause_timer = 0.5;
    }

    fn update(&mut self, state: &mut State) {
        if self.paused {
            return;
        }
        let dt = state.dt;

        if self.wave_active {
            self.wave_time += dt;

            // Check if wave is done
            if self.wave_time >= self.wave_total_duration() {
                // Finalize: set all cells to target hue
                for h in self.cells_hue.iter_mut() {
                    *h = self.target_hue;
                }
                self.current_hue = self.target_hue;
                self.wave_active = false;
                self.pause_timer = 0.0;
            }
        } else {
            self.pause_timer += dt;
            if self.pause_timer >= self.pause_duration {
                self.trigger_wave(state.elapsed);
            }
        }
    }

    fn draw(&mut self, state: &mut State) {
        let half = self.grid_size as f32 / 2.0;

        for gy in 0..self.grid_size {
            for gx in 0..self.grid_size {
                let cx = (gx as f32 - half) + 0.5;
                let cy = (gy as f32 - half) + 0.5;
                let idx = self.cell_index(gx, gy);

                if self.wave_active {
                    let raw_t = self.flip_progress(cx, cy);
                    let eased = Self::bounce_ease(raw_t, self.overshoot);

                    // Color: swap at midpoint
                    let old_hue = self.cells_hue[idx];
                    let hue = if eased >= 0.5 {
                        self.target_hue
                    } else {
                        old_hue
                    };
                    let (r, g, b) = hue_to_rgb(hue);

                    // Rotation value: 0→1 maps one visual period
                    let rot_val = eased;
                    let rotation = match self.wave_rotation {
                        WaveRotation::FlipX => Rotation::FlipX(rot_val),
                        WaveRotation::FlipY => Rotation::FlipY(rot_val),
                        WaveRotation::DiagonalTL => Rotation::DiagonalTL(rot_val),
                        WaveRotation::DiagonalTR => Rotation::DiagonalTR(rot_val),
                        WaveRotation::Z => Rotation::Z(rot_val),
                    };

                    state.draw(
                        Cell::new(cx, cy)
                            .rotation(rotation)
                            .color(Color::linear(r, g, b, 1.0)),
                    );
                } else {
                    let hue = self.cells_hue[idx];
                    let (r, g, b) = hue_to_rgb(hue);
                    state.draw(Cell::new(cx, cy).color(Color::linear(r, g, b, 1.0)));
                }
            }
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit = true,
            // Adjust flip duration
            KeyCode::Key1 => self.flip_duration = (self.flip_duration - 0.1).max(0.1),
            KeyCode::Key2 => self.flip_duration = (self.flip_duration + 0.1).min(2.0),
            // Adjust wave speed
            KeyCode::Key3 => self.delay_per_unit = (self.delay_per_unit - 0.005).max(0.001),
            KeyCode::Key4 => self.delay_per_unit = (self.delay_per_unit + 0.005).min(0.05),
            // Adjust pause
            KeyCode::Key5 => self.pause_duration = (self.pause_duration - 0.1).max(0.0),
            KeyCode::Key6 => self.pause_duration = (self.pause_duration + 0.1).min(3.0),
            // Adjust overshoot
            KeyCode::Key7 => self.overshoot = (self.overshoot - 0.02).max(0.0),
            KeyCode::Key8 => self.overshoot = (self.overshoot + 0.02).min(0.3),
            // Adjust hue step
            KeyCode::Key9 => self.hue_step = (self.hue_step - 0.05).max(0.05),
            KeyCode::Key0 => self.hue_step = (self.hue_step + 0.05).min(0.5),
            // Force trigger
            KeyCode::Space => {
                if !self.wave_active {
                    self.trigger_wave(state.elapsed);
                }
            }
            // Pause/unpause
            KeyCode::P => self.paused = !self.paused,
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let config = Config::builder()
        .title("Rotation Wave Demo")
        .width(900)
        .height(900)
        .viewport(64.0, 64.0)
        .no_file()
        .build();

    tiles::run(RotationDemo::new(), config).unwrap();
}
