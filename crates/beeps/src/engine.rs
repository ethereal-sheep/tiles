use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Stream;

use crate::assets::{AssetId, AssetStore};
use crate::command::{BusId, Command, VoiceId};
use crate::mixer::Mixer;
use crate::modulator::{Adsr, Lfo, LfoWaveform, ModTarget, ModulatorKind};
use crate::source::sample_player::SamplePlayer;
use crate::source::{Oscillator, SourceKind, WavetablePlayer, Waveform};

pub struct AudioEngine {
    producer: rtrb::Producer<Command>,
    next_voice_id: u64,
    next_bus_id: u32,
    sample_rate: u32,
    _stream: Stream,
}

impl AudioEngine {
    pub fn builder() -> AudioEngineBuilder {
        AudioEngineBuilder {
            buffer_size: 512,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn send(&mut self, cmd: Command) {
        let _ = self.producer.push(cmd);
    }

    // --- Voice lifecycle ---

    pub fn play_oscillator(&mut self, waveform: Waveform, frequency: f32, bus: BusId) -> VoiceId {
        let id = self.alloc_voice_id();
        let source = SourceKind::Oscillator(Oscillator::new(waveform, frequency));
        self.send(Command::Play { voice_id: id, source, bus, gain: 1.0, pan: 0.0 });
        id
    }

    pub fn play_wavetable(&mut self, table: Arc<Vec<f32>>, frequency: f32, bus: BusId) -> VoiceId {
        let id = self.alloc_voice_id();
        let source = SourceKind::Wavetable(WavetablePlayer::new(table, frequency));
        self.send(Command::Play { voice_id: id, source, bus, gain: 1.0, pan: 0.0 });
        id
    }

    pub fn play_sample(&mut self, assets: &AssetStore, asset_id: AssetId, bus: BusId) -> Option<VoiceId> {
        let data = assets.get(asset_id)?;
        let id = self.alloc_voice_id();
        let source = SourceKind::SamplePlayer(SamplePlayer::new(data));
        self.send(Command::Play { voice_id: id, source, bus, gain: 1.0, pan: 0.0 });
        Some(id)
    }

    pub fn play_sample_pitched(
        &mut self,
        assets: &AssetStore,
        asset_id: AssetId,
        frequency: f32,
        base_frequency: f32,
        bus: BusId,
    ) -> Option<VoiceId> {
        let data = assets.get(asset_id)?;
        let id = self.alloc_voice_id();
        let mut player = SamplePlayer::new(data);
        player.set_frequency(frequency, base_frequency, self.sample_rate);
        let source = SourceKind::SamplePlayer(player);
        self.send(Command::Play { voice_id: id, source, bus, gain: 1.0, pan: 0.0 });
        Some(id)
    }

    pub fn stop(&mut self, voice: VoiceId) {
        self.send(Command::Stop { voice_id: voice });
    }

    pub fn note_on(&mut self, voice: VoiceId) {
        self.send(Command::NoteOn { voice_id: voice });
    }

    pub fn note_off(&mut self, voice: VoiceId) {
        self.send(Command::NoteOff { voice_id: voice });
    }

    // --- Voice parameters ---

    pub fn set_gain(&mut self, voice: VoiceId, gain: f32) {
        self.send(Command::SetVoiceGain { voice_id: voice, gain });
    }

    pub fn set_pan(&mut self, voice: VoiceId, pan: f32) {
        self.send(Command::SetVoicePan { voice_id: voice, pan: pan.clamp(-1.0, 1.0) });
    }

    pub fn set_pitch(&mut self, voice: VoiceId, frequency: f32) {
        self.send(Command::SetVoicePitch { voice_id: voice, frequency });
    }

    // --- Modulators ---

    pub fn add_adsr(
        &mut self,
        voice: VoiceId,
        target: ModTarget,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) {
        let modulator = ModulatorKind::Adsr(Adsr::new(target, attack, decay, sustain, release));
        self.send(Command::AddModulator { voice_id: voice, modulator, target });
    }

    pub fn add_lfo(
        &mut self,
        voice: VoiceId,
        target: ModTarget,
        waveform: LfoWaveform,
        rate: f32,
        depth: f32,
    ) {
        let modulator = ModulatorKind::Lfo(Lfo::new(target, waveform, rate, depth));
        self.send(Command::AddModulator { voice_id: voice, modulator, target });
    }

    // --- Bus control ---

    pub fn create_bus(&mut self) -> BusId {
        let id = BusId(self.next_bus_id);
        self.next_bus_id += 1;
        self.send(Command::CreateBus { bus_id: id });
        id
    }

    pub fn set_bus_gain(&mut self, bus: BusId, gain: f32) {
        self.send(Command::SetBusGain { bus_id: bus, gain });
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.send(Command::SetMasterGain { gain });
    }

    // --- Internal ---

    fn alloc_voice_id(&mut self) -> VoiceId {
        let id = VoiceId(self.next_voice_id);
        self.next_voice_id += 1;
        id
    }
}

pub struct AudioEngineBuilder {
    buffer_size: u32,
}

impl AudioEngineBuilder {
    pub fn buffer_size(mut self, size: u32) -> Self {
        self.buffer_size = size;
        self
    }

    pub fn build(self) -> Result<AudioEngine, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No audio output device found")?;

        let supported_config = device
            .default_output_config()
            .map_err(|e| e.to_string())?;

        let sample_rate = supported_config.sample_rate().0;
        let channels = supported_config.channels() as usize;

        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Fixed(self.buffer_size),
        };

        let (producer, consumer) = rtrb::RingBuffer::new(1024);
        let mut mixer = Mixer::new();
        let mut consumer = consumer;
        let mut output_buf: Vec<[f32; 2]> = Vec::new();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Drain commands
                    while let Ok(cmd) = consumer.pop() {
                        mixer.process_command(cmd);
                    }

                    let frame_count = data.len() / channels;

                    // Ensure output buffer is large enough
                    if output_buf.len() < frame_count {
                        output_buf.resize(frame_count, [0.0; 2]);
                    }

                    let buf = &mut output_buf[..frame_count];
                    mixer.render(buf, sample_rate);

                    // Write to output (handle mono/stereo/multi-channel)
                    for (i, frame) in buf.iter().enumerate() {
                        let base = i * channels;
                        if base < data.len() {
                            data[base] = frame[0];
                        }
                        if channels >= 2 && base + 1 < data.len() {
                            data[base + 1] = frame[1];
                        }
                        // Zero remaining channels
                        for ch in 2..channels {
                            if base + ch < data.len() {
                                data[base + ch] = 0.0;
                            }
                        }
                    }
                },
                move |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            )
            .map_err(|e| e.to_string())?;

        stream.play().map_err(|e| e.to_string())?;

        Ok(AudioEngine {
            producer,
            next_voice_id: 1,
            next_bus_id: 1, // 0 is master
            sample_rate,
            _stream: stream,
        })
    }
}
