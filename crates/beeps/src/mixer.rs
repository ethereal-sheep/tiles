use crate::command::{BusId, Command, VoiceId};
use crate::effect::{Effect, EffectKind};
use crate::modulator::{ModTarget, Modulator, ModulatorKind};
use crate::source::{ChannelCount, Source, SourceKind, SourceState};

struct ActiveVoice {
    id: VoiceId,
    source: SourceKind,
    bus: BusId,
    gain: f32,
    pan: f32,
    modulators: Vec<ModulatorKind>,
    finished: bool,
}

struct Bus {
    gain: f32,
    effects: Vec<EffectKind>,
    buffer: Vec<[f32; 2]>,
}

pub(crate) struct Mixer {
    voices: Vec<ActiveVoice>,
    buses: Vec<Bus>,
    master_gain: f32,
    sample_counter: u64,
    mono_scratch: Vec<f32>,
    stereo_scratch: Vec<[f32; 2]>,
}

impl Mixer {
    pub fn new() -> Self {
        // Bus 0 is always the master bus
        let master_bus = Bus {
            gain: 1.0,
            effects: Vec::new(),
            buffer: Vec::new(),
        };
        Self {
            voices: Vec::new(),
            buses: vec![master_bus],
            master_gain: 1.0,
            sample_counter: 0,
            mono_scratch: Vec::new(),
            stereo_scratch: Vec::new(),
        }
    }

    pub fn sample_counter(&self) -> u64 {
        self.sample_counter
    }

    pub fn process_command(&mut self, cmd: Command) {
        match cmd {
            Command::Play { voice_id, source, bus, gain, pan } => {
                self.voices.push(ActiveVoice {
                    id: voice_id,
                    source,
                    bus,
                    gain,
                    pan,
                    modulators: Vec::new(),
                    finished: false,
                });
            }
            Command::Stop { voice_id } => {
                self.voices.retain(|v| v.id != voice_id);
            }
            Command::NoteOn { voice_id } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    for m in &mut voice.modulators {
                        m.note_on();
                    }
                }
            }
            Command::NoteOff { voice_id } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    for m in &mut voice.modulators {
                        m.note_off();
                    }
                }
            }
            Command::SetVoiceGain { voice_id, gain } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    voice.gain = gain;
                }
            }
            Command::SetVoicePan { voice_id, pan } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    voice.pan = pan;
                }
            }
            Command::SetVoicePitch { voice_id, frequency } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    match &mut voice.source {
                        SourceKind::Oscillator(osc) => osc.frequency = frequency,
                        SourceKind::Wavetable(wt) => wt.frequency = frequency,
                        SourceKind::SamplePlayer(_) => {}
                    }
                }
            }
            Command::AddModulator { voice_id, modulator, target: _ } => {
                if let Some(voice) = self.voices.iter_mut().find(|v| v.id == voice_id) {
                    voice.modulators.push(modulator);
                }
            }
            Command::CreateBus { bus_id } => {
                let idx = bus_id.0 as usize;
                while self.buses.len() <= idx {
                    self.buses.push(Bus {
                        gain: 1.0,
                        effects: Vec::new(),
                        buffer: Vec::new(),
                    });
                }
            }
            Command::SetBusGain { bus_id, gain } => {
                if let Some(bus) = self.buses.get_mut(bus_id.0 as usize) {
                    bus.gain = gain;
                }
            }
            Command::SetMasterGain { gain } => {
                self.master_gain = gain;
            }
        }
    }

    pub fn render(&mut self, output: &mut [[f32; 2]], sample_rate: u32) {
        let frame_count = output.len();

        // Clear bus buffers
        for bus in &mut self.buses {
            bus.buffer.clear();
            bus.buffer.resize(frame_count, [0.0; 2]);
        }

        // Ensure scratch buffers are large enough
        if self.mono_scratch.len() < frame_count {
            self.mono_scratch.resize(frame_count, 0.0);
        }
        if self.stereo_scratch.len() < frame_count {
            self.stereo_scratch.resize(frame_count, [0.0; 2]);
        }

        // Render each voice into its bus
        for voice in &mut self.voices {
            let bus_idx = voice.bus.0 as usize;
            if bus_idx >= self.buses.len() {
                continue;
            }

            // Render source into scratch buffer
            let state = match voice.source.channel_count() {
                ChannelCount::Mono => {
                    let buf = &mut self.mono_scratch[..frame_count];
                    buf.fill(0.0);
                    let state = voice.source.render(buf, sample_rate);

                    let bus_buf = &mut self.buses[bus_idx].buffer;
                    for i in 0..frame_count {
                        // Advance modulators per sample
                        let mut amp_mod = 1.0f32;
                        let mut pan_mod = 0.0f32;
                        for m in &mut voice.modulators {
                            match m.target() {
                                ModTarget::Amplitude => amp_mod *= m.value(),
                                ModTarget::Pitch => {}
                                ModTarget::Pan => pan_mod += m.value(),
                            }
                            m.advance(sample_rate);
                        }

                        let pan = (voice.pan + pan_mod).clamp(-1.0, 1.0);
                        let gain = voice.gain * amp_mod;
                        let angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
                        let gain_l = angle.cos() * gain;
                        let gain_r = angle.sin() * gain;

                        bus_buf[i][0] += buf[i] * gain_l;
                        bus_buf[i][1] += buf[i] * gain_r;
                    }
                    state
                }
                ChannelCount::Stereo => {
                    let buf = &mut self.stereo_scratch[..frame_count];
                    buf.fill([0.0; 2]);
                    let flat = unsafe {
                        std::slice::from_raw_parts_mut(
                            buf.as_mut_ptr() as *mut f32,
                            frame_count * 2,
                        )
                    };
                    let state = voice.source.render(flat, sample_rate);

                    let bus_buf = &mut self.buses[bus_idx].buffer;
                    for i in 0..frame_count {
                        let mut amp_mod = 1.0f32;
                        let mut pan_mod = 0.0f32;
                        for m in &mut voice.modulators {
                            match m.target() {
                                ModTarget::Amplitude => amp_mod *= m.value(),
                                ModTarget::Pitch => {}
                                ModTarget::Pan => pan_mod += m.value(),
                            }
                            m.advance(sample_rate);
                        }

                        let gain = voice.gain * amp_mod;
                        let balance = (voice.pan + pan_mod).clamp(-1.0, 1.0);
                        let gain_l = gain * (1.0 - balance.max(0.0));
                        let gain_r = gain * (1.0 + balance.min(0.0));

                        bus_buf[i][0] += buf[i][0] * gain_l;
                        bus_buf[i][1] += buf[i][1] * gain_r;
                    }
                    state
                }
            };

            if state == SourceState::Finished {
                voice.finished = true;
            }

            // Check if ADSR amplitude modulator finished
            let has_amp_mod = voice.modulators.iter().any(|m| m.target() == ModTarget::Amplitude);
            if has_amp_mod {
                let amp_done = voice.modulators.iter().all(|m| {
                    m.target() != ModTarget::Amplitude || m.value() == 0.0
                });
                if amp_done {
                    voice.finished = true;
                }
            }
        }

        // Remove finished voices
        self.voices.retain(|v| !v.finished);

        // Apply bus effects and mix to output
        output.fill([0.0; 2]);

        for bus in &mut self.buses {
            // Apply effects
            for effect in &mut bus.effects {
                effect.process(&mut bus.buffer, sample_rate);
            }

            // Sum into output with bus gain
            for (i, frame) in bus.buffer.iter().enumerate() {
                output[i][0] += frame[0] * bus.gain;
                output[i][1] += frame[1] * bus.gain;
            }
        }

        // Apply master gain and clamp
        for frame in output.iter_mut() {
            frame[0] = (frame[0] * self.master_gain).clamp(-1.0, 1.0);
            frame[1] = (frame[1] * self.master_gain).clamp(-1.0, 1.0);
        }

        self.sample_counter += frame_count as u64;
    }
}
