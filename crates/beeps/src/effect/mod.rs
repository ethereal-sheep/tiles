mod gain;
mod filter;
mod delay;
mod bitcrush;

pub use gain::Gain;
pub use filter::{Filter, FilterMode};
pub use delay::Delay;
pub use bitcrush::Bitcrush;

pub trait Effect: Send {
    fn process(&mut self, buffer: &mut [[f32; 2]], sample_rate: u32);
}

pub(crate) enum EffectKind {
    Gain(Gain),
    Filter(Filter),
    Delay(Delay),
    Bitcrush(Bitcrush),
}

impl Effect for EffectKind {
    fn process(&mut self, buffer: &mut [[f32; 2]], sample_rate: u32) {
        match self {
            EffectKind::Gain(e) => e.process(buffer, sample_rate),
            EffectKind::Filter(e) => e.process(buffer, sample_rate),
            EffectKind::Delay(e) => e.process(buffer, sample_rate),
            EffectKind::Bitcrush(e) => e.process(buffer, sample_rate),
        }
    }
}
