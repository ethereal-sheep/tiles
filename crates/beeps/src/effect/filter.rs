use super::Effect;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
}

pub struct Filter {
    pub mode: FilterMode,
    pub cutoff: f32,
    pub resonance: f32,
    // Biquad state per channel
    x1: [f32; 2],
    x2: [f32; 2],
    y1: [f32; 2],
    y2: [f32; 2],
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    last_sample_rate: u32,
}

impl Filter {
    pub fn new(mode: FilterMode, cutoff: f32, resonance: f32) -> Self {
        Self {
            mode,
            cutoff,
            resonance: resonance.max(0.1),
            x1: [0.0; 2],
            x2: [0.0; 2],
            y1: [0.0; 2],
            y2: [0.0; 2],
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            last_sample_rate: 0,
        }
    }

    fn compute_coefficients(&mut self, sample_rate: u32) {
        let omega = std::f32::consts::TAU * self.cutoff / sample_rate as f32;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * self.resonance);

        let (b0, b1, b2, a0, a1, a2) = match self.mode {
            FilterMode::LowPass => {
                let b1 = 1.0 - cos_w;
                let b0 = b1 / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterMode::HighPass => {
                let b1 = -(1.0 + cos_w);
                let b0 = (1.0 + cos_w) / 2.0;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterMode::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }
}

impl Effect for Filter {
    fn process(&mut self, buffer: &mut [[f32; 2]], sample_rate: u32) {
        if sample_rate != self.last_sample_rate {
            self.compute_coefficients(sample_rate);
            self.last_sample_rate = sample_rate;
        }

        for frame in buffer.iter_mut() {
            for ch in 0..2 {
                let input = frame[ch];
                let output = self.b0 * input
                    + self.b1 * self.x1[ch]
                    + self.b2 * self.x2[ch]
                    - self.a1 * self.y1[ch]
                    - self.a2 * self.y2[ch];

                self.x2[ch] = self.x1[ch];
                self.x1[ch] = input;
                self.y2[ch] = self.y1[ch];
                self.y1[ch] = output;

                frame[ch] = output;
            }
        }
    }
}
