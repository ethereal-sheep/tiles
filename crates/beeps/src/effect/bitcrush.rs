use super::Effect;

pub struct Bitcrush {
    pub bit_depth: u32,
    pub rate_reduction: u32,
    hold_l: f32,
    hold_r: f32,
    counter: u32,
}

impl Bitcrush {
    pub fn new(bit_depth: u32, rate_reduction: u32) -> Self {
        Self {
            bit_depth: bit_depth.clamp(1, 16),
            rate_reduction: rate_reduction.max(1),
            hold_l: 0.0,
            hold_r: 0.0,
            counter: 0,
        }
    }

    fn crush(&self, sample: f32) -> f32 {
        let levels = (1 << self.bit_depth) as f32;
        (sample * levels).round() / levels
    }
}

impl Effect for Bitcrush {
    fn process(&mut self, buffer: &mut [[f32; 2]], _sample_rate: u32) {
        for frame in buffer.iter_mut() {
            if self.counter == 0 {
                self.hold_l = self.crush(frame[0]);
                self.hold_r = self.crush(frame[1]);
            }
            self.counter = (self.counter + 1) % self.rate_reduction;

            frame[0] = self.hold_l;
            frame[1] = self.hold_r;
        }
    }
}
