use super::{ModTarget, Modulator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LfoWaveform {
    Sine,
    Square,
    Triangle,
}

#[derive(Clone)]
pub struct Lfo {
    target_param: ModTarget,
    pub waveform: LfoWaveform,
    pub rate: f32,
    pub depth: f32,
    phase: f32,
    current_value: f32,
}

impl Lfo {
    pub fn new(target: ModTarget, waveform: LfoWaveform, rate: f32, depth: f32) -> Self {
        Self {
            target_param: target,
            waveform,
            rate,
            depth,
            phase: 0.0,
            current_value: 0.0,
        }
    }
}

impl Modulator for Lfo {
    fn target(&self) -> ModTarget {
        self.target_param
    }

    fn value(&self) -> f32 {
        self.current_value
    }

    fn advance(&mut self, sample_rate: u32) {
        let phase_inc = self.rate / sample_rate as f32;
        self.phase += phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        let raw = match self.waveform {
            LfoWaveform::Sine => (self.phase * std::f32::consts::TAU).sin(),
            LfoWaveform::Square => {
                if self.phase < 0.5 { 1.0 } else { -1.0 }
            }
            LfoWaveform::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
        };

        self.current_value = raw * self.depth;
    }

    fn note_on(&mut self) {
        self.phase = 0.0;
    }

    fn note_off(&mut self) {}
}
