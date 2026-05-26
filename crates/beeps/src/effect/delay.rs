use super::Effect;

pub struct Delay {
    pub delay_time: f32,
    pub feedback: f32,
    pub mix: f32,
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    last_sample_rate: u32,
}

impl Delay {
    pub fn new(delay_time: f32, feedback: f32, mix: f32) -> Self {
        Self {
            delay_time,
            feedback: feedback.clamp(0.0, 0.95),
            mix: mix.clamp(0.0, 1.0),
            buffer_l: Vec::new(),
            buffer_r: Vec::new(),
            write_pos: 0,
            last_sample_rate: 0,
        }
    }

    fn init_buffers(&mut self, sample_rate: u32) {
        let size = (self.delay_time * sample_rate as f32) as usize;
        let size = size.max(1);
        self.buffer_l = vec![0.0; size];
        self.buffer_r = vec![0.0; size];
        self.write_pos = 0;
    }
}

impl Effect for Delay {
    fn process(&mut self, buffer: &mut [[f32; 2]], sample_rate: u32) {
        if sample_rate != self.last_sample_rate {
            self.init_buffers(sample_rate);
            self.last_sample_rate = sample_rate;
        }

        let buf_len = self.buffer_l.len();
        if buf_len == 0 {
            return;
        }

        for frame in buffer.iter_mut() {
            let read_pos = self.write_pos;
            let delayed_l = self.buffer_l[read_pos];
            let delayed_r = self.buffer_r[read_pos];

            self.buffer_l[self.write_pos] = frame[0] + delayed_l * self.feedback;
            self.buffer_r[self.write_pos] = frame[1] + delayed_r * self.feedback;

            self.write_pos = (self.write_pos + 1) % buf_len;

            frame[0] = frame[0] * (1.0 - self.mix) + delayed_l * self.mix;
            frame[1] = frame[1] * (1.0 - self.mix) + delayed_r * self.mix;
        }
    }
}
