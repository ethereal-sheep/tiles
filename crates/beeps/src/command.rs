use crate::modulator::{ModTarget, ModulatorKind};
use crate::source::SourceKind;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VoiceId(pub(crate) u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BusId(pub(crate) u32);

impl BusId {
    pub const MASTER: BusId = BusId(0);
}

pub(crate) enum Command {
    // Voice lifecycle
    Play {
        voice_id: VoiceId,
        source: SourceKind,
        bus: BusId,
        gain: f32,
        pan: f32,
    },
    Stop {
        voice_id: VoiceId,
    },
    NoteOn {
        voice_id: VoiceId,
    },
    NoteOff {
        voice_id: VoiceId,
    },

    // Voice parameters
    SetVoiceGain {
        voice_id: VoiceId,
        gain: f32,
    },
    SetVoicePan {
        voice_id: VoiceId,
        pan: f32,
    },
    SetVoicePitch {
        voice_id: VoiceId,
        frequency: f32,
    },

    // Modulators
    AddModulator {
        voice_id: VoiceId,
        modulator: ModulatorKind,
        target: ModTarget,
    },

    // Bus control
    CreateBus {
        bus_id: BusId,
    },
    SetBusGain {
        bus_id: BusId,
        gain: f32,
    },
    SetMasterGain {
        gain: f32,
    },
}
