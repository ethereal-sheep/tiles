use std::sync::Arc;

use super::{ChannelCount, Source, SourceState};

#[derive(Clone)]
pub struct WavetablePlayer {
    table: Arc<Vec<f32>>,
    pub frequency: f32,
    pub amplitude: f32,
    phase: f64,
}

impl WavetablePlayer {
    pub fn new(table: Arc<Vec<f32>>, frequency: f32) -> Self {
        Self {
            table,
            frequency,
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude;
        self
    }
}

impl Source for WavetablePlayer {
    fn channel_count(&self) -> ChannelCount {
        ChannelCount::Mono
    }

    fn render(&mut self, buffer: &mut [f32], sample_rate: u32) -> SourceState {
        let table_len = self.table.len() as f64;
        let phase_inc = self.frequency as f64 * table_len / sample_rate as f64;

        for sample in buffer.iter_mut() {
            let idx = self.phase as usize;
            let frac = (self.phase - idx as f64) as f32;
            let s0 = self.table[idx % self.table.len()];
            let s1 = self.table[(idx + 1) % self.table.len()];
            *sample = (s0 + (s1 - s0) * frac) * self.amplitude;

            self.phase += phase_inc;
            if self.phase >= table_len {
                self.phase -= table_len;
            }
        }
        SourceState::Playing
    }
}
