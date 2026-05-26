use super::Effect;

pub struct Gain {
    pub value: f32,
}

impl Gain {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl Effect for Gain {
    fn process(&mut self, buffer: &mut [[f32; 2]], _sample_rate: u32) {
        for frame in buffer.iter_mut() {
            frame[0] *= self.value;
            frame[1] *= self.value;
        }
    }
}
