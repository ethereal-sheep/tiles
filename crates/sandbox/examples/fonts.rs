use tiles::{
    AnchorCorner::TopLeft, App, Color, Config, Drawable, KeyCode, KeyEvent, KeyState, MouseEvent, Shape, State, StrokePosition::Middle, Text, font::{
        Font, HELVR12_7X9, LOGISOSO46_23X46, MONO_4X6, MONO_5X7, MONO_5X8, MONO_6X9, MONO_6X10, MONO_6X12, MONO_6X13, MONO_7X13, MONO_7X14, MONO_8X13, MONO_9X15, MONO_9X18, MONO_10X20, MONO_BOLD_6X13, MONO_BOLD_7X13, MONO_BOLD_7X14, MONO_BOLD_8X13, MONO_BOLD_9X15, MONO_BOLD_9X18, MONO_CLR6X12_6X12, MONO_CREEP_4X7, MONO_HAXORMEDIUM_6X11, MONO_HAXORMEDIUM_7X13, MONO_HAXORMEDIUM_8X14, MONO_HAXORMEDIUM_9X15, MONO_HAXORNARROW_5X11, MONO_HAXORNARROW_6X12, MONO_HAXORNARROW_7X13, MONO_KNXT_9X20, MONO_OBLIQUE_6X13, MONO_OBLIQUE_7X13, MONO_OBLIQUE_8X13, MONO_PEEP_10X21, MONO_PSEVDOAZBUKAMEDIUM_8X14, MONO_SCIENTIFICA_4X7, MONO_SCIENTIFICABOLD_4X7, MONO_SCIENTIFICAITALIC_4X7, MONO_SPLEEN_5X8, MONO_SPLEEN_8X16, MONO_SPLEEN_12X24, MONO_SPLEEN_16X32, MONO_SPLEEN_32X64, TINY5_4X5, TOM_THUMB_3X5
    }
};

struct FontDemo {
    font_index: usize,
}

const FONTS: &[(&str, &Font)] = &[
    ("Tom Thumb 3x5", &TOM_THUMB_3X5),
    ("Tiny5 4x5", &TINY5_4X5),
    ("4x6", &MONO_4X6),
    ("Creep 4x7", &MONO_CREEP_4X7),
    ("Scientifica 4x7", &MONO_SCIENTIFICA_4X7),
    ("Scientifica Bold 4x7", &MONO_SCIENTIFICABOLD_4X7),
    ("Scientifica Italic 4x7", &MONO_SCIENTIFICAITALIC_4X7),
    ("5x7", &MONO_5X7),
    ("5x8", &MONO_5X8),
    ("Spleen 5x8", &MONO_SPLEEN_5X8),
    ("6x9", &MONO_6X9),
    ("6x10", &MONO_6X10),
    ("6x12", &MONO_6X12),
    ("6x13", &MONO_6X13),
    ("Bold 6x13", &MONO_BOLD_6X13),
    ("Oblique 6x13", &MONO_OBLIQUE_6X13),
    ("clR 6x12", &MONO_CLR6X12_6X12),
    ("7x13", &MONO_7X13),
    ("Bold 7x13", &MONO_BOLD_7X13),
    ("Oblique 7x13", &MONO_OBLIQUE_7X13),
    ("7x14", &MONO_7X14),
    ("Bold 7x14", &MONO_BOLD_7X14),
    ("8x13", &MONO_8X13),
    ("Bold 8x13", &MONO_BOLD_8X13),
    ("Oblique 8x13", &MONO_OBLIQUE_8X13),
    ("Spleen 8x16", &MONO_SPLEEN_8X16),
    ("9x15", &MONO_9X15),
    ("Bold 9x15", &MONO_BOLD_9X15),
    ("9x18", &MONO_9X18),
    ("Bold 9x18", &MONO_BOLD_9X18),
    ("10x20", &MONO_10X20),
    ("Peep 10x21", &MONO_PEEP_10X21),
    ("Haxor Medium 6x11", &MONO_HAXORMEDIUM_6X11),
    ("Haxor Medium 7x13", &MONO_HAXORMEDIUM_7X13),
    ("Haxor Medium 8x14", &MONO_HAXORMEDIUM_8X14),
    ("Haxor Medium 9x15", &MONO_HAXORMEDIUM_9X15),
    ("Haxor Narrow 5x11", &MONO_HAXORNARROW_5X11),
    ("Haxor Narrow 6x12", &MONO_HAXORNARROW_6X12),
    ("Haxor Narrow 7x13", &MONO_HAXORNARROW_7X13),
    ("Helv 7x9", &HELVR12_7X9),
    ("Knxt 9x20", &MONO_KNXT_9X20),
    ("Psevdo Azbuka 8x14", &MONO_PSEVDOAZBUKAMEDIUM_8X14),
    ("Spleen 12x24", &MONO_SPLEEN_12X24),
    ("Spleen 16x32", &MONO_SPLEEN_16X32),
    ("Spleen 32x64", &MONO_SPLEEN_32X64),
    ("Logisoso 23x46", &LOGISOSO46_23X46),
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
        Self { font_index: 0 }
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
        let text_width = max_chars as f32 * font.char_advance('A') as f32;
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
        let start_x = 10.0;
        let start_y = 10.0;

        let header = format!("[{}/{}]", self.font_index + 1, FONTS.len());
        let test = Text::new(font, "Hello World").position(start_x, start_y - line_height).anchor(tiles::AnchorBox::Tight, TopLeft);
        let bounds = test.bounds().offset(1);
        state.draw_screen(bounds.stroke(2, Middle).color(Color::linear(1.0, 1.0, 1.0, 1.0)));
        state.draw_screen(test.color(Color::linear(0.1, 0.1, 0.1, 1.0)));
        state.draw_screen(Text::new(font, &header).position(start_x, start_y));
        state.draw_screen(Text::new(font, name).position(start_x, start_y + line_height));

        for (i, line) in SAMPLE_LINES.iter().enumerate() {
            let y = start_y + line_height * (i as f32 + 2.0);
            state.draw_screen(Text::new(font, *line).position(start_x, y));
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
