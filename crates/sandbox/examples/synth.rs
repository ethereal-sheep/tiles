use std::collections::HashMap;

use beeps::modulator::ModTarget;
use beeps::source::Waveform;
use beeps::{AudioEngine, BusId, VoiceId};
use tiles::font::MONO_5X7;
use tiles::{App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, State};

struct SynthDemo {
    engine: AudioEngine,
    bus: BusId,
    octave: i32,
    waveform: Waveform,
    waveform_idx: usize,
    held_voices: HashMap<KeyCode, VoiceId>,
}

const WAVEFORMS: &[(Waveform, &str)] = &[
    (Waveform::Square, "Square"),
    (Waveform::Pulse, "Pulse"),
    (Waveform::Sawtooth, "Sawtooth"),
    (Waveform::Triangle, "Triangle"),
    (Waveform::Sine, "Sine"),
    (Waveform::Noise, "Noise"),
];

const KEY_MAP: &[(KeyCode, i32)] = &[
    (KeyCode::Z, 0),   // C
    (KeyCode::S, 1),   // C#
    (KeyCode::X, 2),   // D
    (KeyCode::D, 3),   // D#
    (KeyCode::C, 4),   // E
    (KeyCode::V, 5),   // F
    (KeyCode::G, 6),   // F#
    (KeyCode::B, 7),   // G
    (KeyCode::H, 8),   // G#
    (KeyCode::N, 9),   // A
    (KeyCode::J, 10),  // A#
    (KeyCode::M, 11),  // B
];

impl SynthDemo {
    fn new() -> Self {
        let mut engine = AudioEngine::builder().buffer_size(256).build().unwrap();
        let bus = engine.create_bus();
        engine.set_bus_gain(bus, 0.15);
        Self {
            engine,
            bus,
            octave: 4,
            waveform: Waveform::Square,
            waveform_idx: 0,
            held_voices: HashMap::new(),
        }
    }

    fn note_to_freq(note: i32, octave: i32) -> f32 {
        let midi = (octave + 1) * 12 + note;
        440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
    }

    fn play_note(&mut self, key: KeyCode, semitone: i32) {
        if self.held_voices.contains_key(&key) {
            return;
        }
        let freq = Self::note_to_freq(semitone, self.octave);
        let voice = self.engine.play_oscillator(self.waveform, freq, self.bus);
        self.engine.add_adsr(voice, ModTarget::Amplitude, 0.01, 0.05, 0.7, 0.15);
        self.engine.note_on(voice);
        self.held_voices.insert(key, voice);
    }

    fn release_note(&mut self, key: KeyCode) {
        if let Some(voice) = self.held_voices.remove(&key) {
            self.engine.note_off(voice);
        }
    }

    fn draw_text(state: &mut State, text: &str, x: f32, y: f32, color: [f32; 4]) {
        let font = &MONO_5X7;
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(glyph) = font.glyph(ch) {
                for row in 0..font.height {
                    for col in 0..font.glyph_width(ch) {
                        if font.pixel(glyph, col, row) {
                            state.draw(
                                Cell::new(cursor_x + col as f32, y - row as f32)
                                    .color(Color::linear(color[0], color[1], color[2], color[3])),
                            );
                        }
                    }
                }
            }
            cursor_x += font.char_advance(ch) as f32;
        }
    }
}

impl App for SynthDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.08, 0.08, 0.12, 1.0);
        state.set_window_background(0.02, 0.02, 0.04, 1.0);
        state.set_ambient_illumination(1.0);
    }

    fn draw(&mut self, state: &mut State) {
        let vp = state.viewport_size();
        let base_x = -vp.x / 2.0 + 4.0;
        let top_y = vp.y / 2.0 - 4.0;

        Self::draw_text(state, "SYNTH DEMO", base_x, top_y, [1.0, 1.0, 1.0, 1.0]);

        let wf_name = WAVEFORMS[self.waveform_idx].1;
        let label = format!("Waveform: {} (Left/Right)", wf_name);
        Self::draw_text(state, &label, base_x, top_y - 12.0, [0.8, 0.8, 0.5, 1.0]);

        let oct_label = format!("Octave: {} (Up/Down)", self.octave);
        Self::draw_text(state, &oct_label, base_x, top_y - 22.0, [0.5, 0.8, 0.8, 1.0]);

        Self::draw_text(state, "Keys: Z S X D C V G B H N J M", base_x, top_y - 36.0, [0.6, 0.6, 0.6, 1.0]);
        Self::draw_text(state, "      C   D   E F   G   A   B", base_x, top_y - 46.0, [0.5, 0.5, 0.5, 1.0]);

        // Piano visualization
        let piano_y = -10.0;
        let piano_x = -30.0;
        let white_notes = [0, 2, 4, 5, 7, 9, 11];
        let black_notes = [1, 3, -1, 6, 8, 10, -1];

        for (i, _note) in white_notes.iter().enumerate() {
            let kx = piano_x + i as f32 * 5.0;
            for dy in 0..8 {
                for dx in 0..4 {
                    state.draw(Cell::new(kx + dx as f32, piano_y - dy as f32).color(Color::linear(0.9, 0.9, 0.9, 1.0)));
                }
            }
        }

        for (i, note) in black_notes.iter().enumerate() {
            if *note < 0 {
                continue;
            }
            let kx = piano_x + i as f32 * 5.0 + 3.0;
            for dy in 0..5 {
                for dx in 0..3 {
                    state.draw(Cell::new(kx + dx as f32, piano_y - dy as f32).color(Color::linear(0.15, 0.15, 0.15, 1.0)));
                }
            }
        }

        Self::draw_text(state, "ESC to quit", base_x, -vp.y / 2.0 + 7.0, [0.4, 0.4, 0.4, 1.0]);
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        match event.state {
            KeyState::Pressed => match event.key {
                KeyCode::Escape => state.quit = true,
                KeyCode::Left => {
                    if self.waveform_idx > 0 {
                        self.waveform_idx -= 1;
                        self.waveform = WAVEFORMS[self.waveform_idx].0;
                    }
                }
                KeyCode::Right => {
                    if self.waveform_idx + 1 < WAVEFORMS.len() {
                        self.waveform_idx += 1;
                        self.waveform = WAVEFORMS[self.waveform_idx].0;
                    }
                }
                KeyCode::Up => {
                    self.octave = (self.octave + 1).min(7);
                }
                KeyCode::Down => {
                    self.octave = (self.octave - 1).max(1);
                }
                other => {
                    for (key, semitone) in KEY_MAP {
                        if other == *key {
                            self.play_note(*key, *semitone);
                            break;
                        }
                    }
                }
            },
            KeyState::Released => {
                self.release_note(event.key);
            }
        }
    }
}

fn main() {
    let config = Config::builder()
        .title("Synth Demo")
        .width(600)
        .height(600)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    tiles::run(SynthDemo::new(), config).unwrap();
}
