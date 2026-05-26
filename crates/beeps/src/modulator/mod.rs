mod adsr;
mod lfo;

pub use adsr::Adsr;
pub use lfo::{Lfo, LfoWaveform};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ModTarget {
    Amplitude,
    Pitch,
    Pan,
}

pub trait Modulator: Send {
    fn target(&self) -> ModTarget;
    fn value(&self) -> f32;
    fn advance(&mut self, sample_rate: u32);
    fn note_on(&mut self);
    fn note_off(&mut self);
}

pub(crate) enum ModulatorKind {
    Adsr(Adsr),
    Lfo(Lfo),
}

impl Modulator for ModulatorKind {
    fn target(&self) -> ModTarget {
        match self {
            ModulatorKind::Adsr(m) => m.target(),
            ModulatorKind::Lfo(m) => m.target(),
        }
    }

    fn value(&self) -> f32 {
        match self {
            ModulatorKind::Adsr(m) => m.value(),
            ModulatorKind::Lfo(m) => m.value(),
        }
    }

    fn advance(&mut self, sample_rate: u32) {
        match self {
            ModulatorKind::Adsr(m) => m.advance(sample_rate),
            ModulatorKind::Lfo(m) => m.advance(sample_rate),
        }
    }

    fn note_on(&mut self) {
        match self {
            ModulatorKind::Adsr(m) => m.note_on(),
            ModulatorKind::Lfo(m) => m.note_on(),
        }
    }

    fn note_off(&mut self) {
        match self {
            ModulatorKind::Adsr(m) => m.note_off(),
            ModulatorKind::Lfo(m) => m.note_off(),
        }
    }
}
