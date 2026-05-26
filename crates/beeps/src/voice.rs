use std::sync::Arc;

use crate::assets::{AssetId, AssetStore};
use crate::command::{BusId, VoiceId};
use crate::engine::AudioEngine;
use crate::source::Waveform;

pub struct VoicePool {
    bus: BusId,
}

impl VoicePool {
    pub fn new(engine: &mut AudioEngine) -> Self {
        let bus = engine.create_bus();
        Self { bus }
    }

    pub fn with_bus(bus: BusId) -> Self {
        Self { bus }
    }

    pub fn play_oscillator(&self, engine: &mut AudioEngine, waveform: Waveform, frequency: f32) -> VoiceId {
        engine.play_oscillator(waveform, frequency, self.bus)
    }

    pub fn play_wavetable(&self, engine: &mut AudioEngine, table: Arc<Vec<f32>>, frequency: f32) -> VoiceId {
        engine.play_wavetable(table, frequency, self.bus)
    }

    pub fn play_sample(&self, engine: &mut AudioEngine, assets: &AssetStore, asset_id: AssetId) -> Option<VoiceId> {
        engine.play_sample(assets, asset_id, self.bus)
    }

    pub fn bus(&self) -> BusId {
        self.bus
    }
}
