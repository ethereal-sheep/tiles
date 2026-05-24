use tiles::{
    font::{
        Font, CLR6X12, CREEP, FONT_10X20, FONT_4X6, FONT_5X7, FONT_5X8, FONT_6X10, FONT_6X12,
        FONT_6X13, FONT_6X13B, FONT_6X13O, FONT_6X9, FONT_7X13, FONT_7X13B, FONT_7X13O,
        FONT_7X14, FONT_7X14B, FONT_8X13, FONT_8X13B, FONT_8X13O, FONT_9X15, FONT_9X15B,
        FONT_9X18, FONT_9X18B, HAXORMEDIUM_10, HAXORMEDIUM_11, HAXORMEDIUM_12, HAXORMEDIUM_13,
        HAXORNARROW_15, HAXORNARROW_16, HAXORNARROW_17, HELVR12, KNXT, LOGISOSO46, PEEP_10X20,
        PSEVDOAZBUKAMEDIUM_12, SCIENTIFICA_11, SCIENTIFICABOLD_11, SCIENTIFICAITALIC_11,
        SPLEEN_12X24, SPLEEN_16X32, SPLEEN_32X64, SPLEEN_5X8, SPLEEN_8X16, TOM_THUMB,
    },
    App, Cell, Config, KeyCode, KeyEvent, KeyState, MouseEvent, State,
};

struct FontDemo {
    font_index: usize,
    illuminated: bool,
}

const FONTS: &[(&str, &Font)] = &[
    ("Tom Thumb", &TOM_THUMB),
    ("4x6", &FONT_4X6),
    ("5x7", &FONT_5X7),
    ("5x8", &FONT_5X8),
    ("Spleen 5x8", &SPLEEN_5X8),
    ("6x9", &FONT_6X9),
    ("6x10", &FONT_6X10),
    ("6x12", &FONT_6X12),
    ("6x13", &FONT_6X13),
    ("6x13B", &FONT_6X13B),
    ("6x13O", &FONT_6X13O),
    ("clR6x12", &CLR6X12),
    ("7x13", &FONT_7X13),
    ("7x13B", &FONT_7X13B),
    ("7x13O", &FONT_7X13O),
    ("7x14", &FONT_7X14),
    ("7x14B", &FONT_7X14B),
    ("8x13", &FONT_8X13),
    ("8x13B", &FONT_8X13B),
    ("8x13O", &FONT_8X13O),
    ("Spleen 8x16", &SPLEEN_8X16),
    ("9x15", &FONT_9X15),
    ("9x15B", &FONT_9X15B),
    ("9x18", &FONT_9X18),
    ("9x18B", &FONT_9X18B),
    ("10x20", &FONT_10X20),
    ("Peep 10x20", &PEEP_10X20),
    ("Creep", &CREEP),
    ("Scientifica 11", &SCIENTIFICA_11),
    ("Scientifica Bold 11", &SCIENTIFICABOLD_11),
    ("Scientifica Italic 11", &SCIENTIFICAITALIC_11),
    ("Haxor Medium 10", &HAXORMEDIUM_10),
    ("Haxor Medium 11", &HAXORMEDIUM_11),
    ("Haxor Medium 12", &HAXORMEDIUM_12),
    ("Haxor Medium 13", &HAXORMEDIUM_13),
    ("Haxor Narrow 15", &HAXORNARROW_15),
    ("Haxor Narrow 16", &HAXORNARROW_16),
    ("Haxor Narrow 17", &HAXORNARROW_17),
    ("Helv R12", &HELVR12),
    ("Knxt", &KNXT),
    ("Psevdo Azbuka 12", &PSEVDOAZBUKAMEDIUM_12),
    ("Spleen 12x24", &SPLEEN_12X24),
    ("Spleen 16x32", &SPLEEN_16X32),
    ("Spleen 32x64", &SPLEEN_32X64),
    ("Logisoso 46", &LOGISOSO46),
];

const SAMPLE_LINES: &[&str] = &[
    "ABCDEFGHIJ",
    "KLMNOPQRST",
    "UVWXYZabcd",
    "efghijklmn",
    "opqrstuvwx",
    "yz01234567",
    "89!@#$%^&*",
    "The quick brown",
    "fox jumps over",
    "the lazy dog.",
];

impl FontDemo {
    fn new() -> Self {
        Self { font_index: 0, illuminated: false }
    }

    fn draw_string(state: &mut State, text: &str, font: &Font, start_x: f32, start_y: f32, illuminated: bool) {
        let mut cursor_x = start_x;
        for ch in text.chars() {
            if let Some(glyph) = font.glyph(ch) {
                for row in 0..font.height {
                    for col in 0..font.width {
                        if font.pixel(glyph, col, row) {
                            let x = cursor_x + col as f32;
                            let y = start_y - row as f32;
                            let cell = Cell::new(x, y).rgba(1.0, 1.0, 1.0, 1.0);
                            state.draw(if illuminated { cell } else { cell.light(1.0) });
                        }
                    }
                }
            }
            cursor_x += font.char_advance() as f32;
        }
    }
}

impl App for FontDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.05, 0.05, 0.08, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(0.0);
    }

    fn update(&mut self, state: &mut State) {
        let (_, font) = FONTS[self.font_index];
        let max_chars = SAMPLE_LINES.iter().map(|l| l.len()).max().unwrap_or(10);
        let text_width = max_chars as f32 * font.char_advance() as f32;
        let line_height = font.height as f32 + 2.0;
        let text_height = (SAMPLE_LINES.len() as f32 + 2.0) * line_height;

        let vp_w = (text_width + 20.0).max(128.0);
        let vp_h = (text_height + 20.0).max(128.0);
        state.set_viewport(vp_w, vp_h);

        let pixels_per_cell = 4u32;
        let win_w = vp_w as u32 * pixels_per_cell;
        let win_h = vp_h as u32 * pixels_per_cell;
        state.set_window_size(win_w.min(1920), win_h.min(1080));
    }

    fn draw(&mut self, state: &mut State) {
        let (name, font) = FONTS[self.font_index];

        let line_height = font.height as f32 + 2.0;
        let vp = state.viewport_size();
        let start_x = -vp.x / 2.0 + 10.0;
        let start_y = vp.y / 2.0 - 10.0;

        // Draw font name and index as header
        let header = format!("[{}/{}] {}", self.font_index + 1, FONTS.len(), name);
        FontDemo::draw_string(state, &header, font, start_x, start_y, self.illuminated);

        // Draw sample lines
        for (i, line) in SAMPLE_LINES.iter().enumerate() {
            let y = start_y - line_height * (i as f32 + 2.0);
            FontDemo::draw_string(state, line, font, start_x, y, self.illuminated);
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state != KeyState::Pressed {
            return;
        }
        match event.key {
            KeyCode::Escape => state.quit = true,
            KeyCode::Right => self.font_index = (self.font_index + 1) % FONTS.len(),
            KeyCode::Left => {
                self.font_index = if self.font_index == 0 {
                    FONTS.len() - 1
                } else {
                    self.font_index - 1
                };
            }
            KeyCode::L => {
                self.illuminated = !self.illuminated;
                let ambient = if self.illuminated { 0.3 } else { 0.0 };
                state.set_ambient_illumination(ambient);
            }
            _ => {}
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let config = Config::builder()
        .title("Fonts")
        .width(1200)
        .height(800)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    tiles::run(FontDemo::new(), config).unwrap();
}
