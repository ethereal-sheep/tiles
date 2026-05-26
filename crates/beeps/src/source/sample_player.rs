use std::sync::Arc;

use super::{ChannelCount, Source, SourceState};

#[derive(Clone)]
pub struct SamplePlayer {
    data: Arc<SampleData>,
    cursor: f64,
    playback_rate: f64,
    looping: bool,
    loop_start: usize,
    loop_end: usize,
    channel_count: ChannelCount,
}

pub struct SampleData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl SamplePlayer {
    pub fn new(data: Arc<SampleData>) -> Self {
        let channel_count = if data.channels == 2 {
            ChannelCount::Stereo
        } else {
            ChannelCount::Mono
        };
        let frame_count = data.samples.len() / data.channels as usize;
        Self {
            data,
            cursor: 0.0,
            playback_rate: 1.0,
            looping: false,
            loop_start: 0,
            loop_end: frame_count,
            channel_count,
        }
    }

    pub fn with_rate(mut self, rate: f64) -> Self {
        self.playback_rate = rate;
        self
    }

    pub fn with_loop(mut self, start: usize, end: usize) -> Self {
        self.looping = true;
        self.loop_start = start;
        self.loop_end = end;
        self
    }

    pub fn set_frequency(&mut self, frequency: f32, base_frequency: f32, device_rate: u32) {
        self.playback_rate = (frequency as f64 / base_frequency as f64)
            * (self.data.sample_rate as f64 / device_rate as f64);
    }

    fn frame_count(&self) -> usize {
        self.data.samples.len() / self.data.channels as usize
    }

    fn interpolate_mono(&self, pos: f64) -> f32 {
        let idx = pos as usize;
        let frac = (pos - idx as f64) as f32;
        let s0 = self.data.samples.get(idx).copied().unwrap_or(0.0);
        let s1 = self.data.samples.get(idx + 1).copied().unwrap_or(s0);
        s0 + (s1 - s0) * frac
    }

    fn interpolate_stereo(&self, pos: f64) -> [f32; 2] {
        let idx = (pos as usize) * 2;
        let frac = (pos - pos.floor()) as f32;
        let l0 = self.data.samples.get(idx).copied().unwrap_or(0.0);
        let r0 = self.data.samples.get(idx + 1).copied().unwrap_or(0.0);
        let l1 = self.data.samples.get(idx + 2).copied().unwrap_or(l0);
        let r1 = self.data.samples.get(idx + 3).copied().unwrap_or(r0);
        [l0 + (l1 - l0) * frac, r0 + (r1 - r0) * frac]
    }
}

impl Source for SamplePlayer {
    fn channel_count(&self) -> ChannelCount {
        self.channel_count
    }

    fn render(&mut self, buffer: &mut [f32], sample_rate: u32) -> SourceState {
        let rate_adjust = self.data.sample_rate as f64 / sample_rate as f64;
        let effective_rate = self.playback_rate * rate_adjust;
        let frames = self.frame_count();

        match self.channel_count {
            ChannelCount::Mono => {
                for sample in buffer.iter_mut() {
                    if self.cursor >= frames as f64 {
                        if self.looping {
                            self.cursor = self.loop_start as f64;
                        } else {
                            *sample = 0.0;
                            return SourceState::Finished;
                        }
                    }
                    *sample = self.interpolate_mono(self.cursor);
                    self.cursor += effective_rate;

                    if self.looping && self.cursor >= self.loop_end as f64 {
                        self.cursor = self.loop_start as f64;
                    }
                }
            }
            ChannelCount::Stereo => {
                for chunk in buffer.chunks_exact_mut(2) {
                    if self.cursor >= frames as f64 {
                        if self.looping {
                            self.cursor = self.loop_start as f64;
                        } else {
                            chunk[0] = 0.0;
                            chunk[1] = 0.0;
                            return SourceState::Finished;
                        }
                    }
                    let [l, r] = self.interpolate_stereo(self.cursor);
                    chunk[0] = l;
                    chunk[1] = r;
                    self.cursor += effective_rate;

                    if self.looping && self.cursor >= self.loop_end as f64 {
                        self.cursor = self.loop_start as f64;
                    }
                }
            }
        }
        SourceState::Playing
    }
}
