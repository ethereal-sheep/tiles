use super::{ChannelCount, Source, SourceState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Pulse,
    Triangle,
    Sawtooth,
    Noise,
}

#[derive(Clone)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub frequency: f32,
    pub amplitude: f32,
    pub duty_cycle: f32,
    phase: f32,
    noise_state: u32,
}

impl Oscillator {
    pub fn new(waveform: Waveform, frequency: f32) -> Self {
        let duty_cycle = match waveform {
            Waveform::Pulse => 0.25,
            _ => 0.5,
        };
        Self {
            waveform,
            frequency,
            amplitude: 1.0,
            duty_cycle,
            phase: 0.0,
            noise_state: 0xACE1,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn with_duty_cycle(mut self, duty: f32) -> Self {
        self.duty_cycle = duty.clamp(0.01, 0.99);
        self
    }

    fn polyblep(phase: f32, phase_inc: f32) -> f32 {
        if phase < phase_inc {
            let t = phase / phase_inc;
            2.0 * t - t * t - 1.0
        } else if phase > 1.0 - phase_inc {
            let t = (phase - 1.0) / phase_inc;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }

    fn generate_sample(&mut self, phase_inc: f32) -> f32 {
        let sample = match self.waveform {
            Waveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            Waveform::Square => {
                let mut s = if self.phase < 0.5 { 1.0 } else { -1.0 };
                s += Self::polyblep(self.phase, phase_inc);
                s -= Self::polyblep((self.phase + 0.5) % 1.0, phase_inc);
                s
            }
            Waveform::Pulse => {
                let mut s = if self.phase < self.duty_cycle { 1.0 } else { -1.0 };
                s += Self::polyblep(self.phase, phase_inc);
                s -= Self::polyblep((self.phase - self.duty_cycle + 1.0) % 1.0, phase_inc);
                s
            }
            Waveform::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
            Waveform::Sawtooth => {
                let mut s = 2.0 * self.phase - 1.0;
                s -= Self::polyblep(self.phase, phase_inc);
                s
            }
            Waveform::Noise => {
                let bit = ((self.noise_state >> 0) ^ (self.noise_state >> 1)) & 1;
                self.noise_state = (self.noise_state >> 1) | (bit << 14);
                (self.noise_state & 1) as f32 * 2.0 - 1.0
            }
        };
        sample * self.amplitude
    }
}

impl Source for Oscillator {
    fn channel_count(&self) -> ChannelCount {
        ChannelCount::Mono
    }

    fn render(&mut self, buffer: &mut [f32], sample_rate: u32) -> SourceState {
        let phase_inc = self.frequency / sample_rate as f32;
        for sample in buffer.iter_mut() {
            *sample = self.generate_sample(phase_inc);
            self.phase += phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
        }
        SourceState::Playing
    }
}
