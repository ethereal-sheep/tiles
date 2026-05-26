mod oscillator;
pub(crate) mod sample_player;
mod wavetable;

pub use oscillator::{Oscillator, Waveform};
pub use sample_player::SamplePlayer;
pub use wavetable::WavetablePlayer;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SourceState {
    Playing,
    Finished,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChannelCount {
    Mono,
    Stereo,
}

pub trait Source: Send {
    fn channel_count(&self) -> ChannelCount;
    fn render(&mut self, buffer: &mut [f32], sample_rate: u32) -> SourceState;
}

pub(crate) enum SourceKind {
    Oscillator(Oscillator),
    SamplePlayer(SamplePlayer),
    Wavetable(WavetablePlayer),
}

impl Source for SourceKind {
    fn channel_count(&self) -> ChannelCount {
        match self {
            SourceKind::Oscillator(s) => s.channel_count(),
            SourceKind::SamplePlayer(s) => s.channel_count(),
            SourceKind::Wavetable(s) => s.channel_count(),
        }
    }

    fn render(&mut self, buffer: &mut [f32], sample_rate: u32) -> SourceState {
        match self {
            SourceKind::Oscillator(s) => s.render(buffer, sample_rate),
            SourceKind::SamplePlayer(s) => s.render(buffer, sample_rate),
            SourceKind::Wavetable(s) => s.render(buffer, sample_rate),
        }
    }
}
