use tiles::{
    App, Color, ColorScale, Config, Drawable, KeyCode, KeyEvent, KeyState, MouseEvent, Rect, Shape,
    State, Text, Theme, font::MONO_4X6,
};

struct ColorDemo {
    theme: Theme,
}

impl ColorDemo {
    fn new() -> Self {
        Self { theme: Theme::default() }
    }

    fn draw_scale(&self, state: &mut State, scale: &ColorScale, x: f32, y: f32) {
        let variants = [
            scale.darker,
            scale.dark,
            scale.base,
            scale.light,
            scale.lighter,
        ];
        for (i, color) in variants.iter().enumerate() {
            let sx = x + i as f32 * 6.0;
            state.draw_screen(
                Rect::from_top_left(sx, y, 5, 5)
                    .fill()
                    .color(*color),
            );
        }
    }
}

impl App for ColorDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.1, 0.1, 0.12, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(1.0);
    }

    fn update(&mut self, _state: &mut State) {}

    fn draw(&mut self, state: &mut State) {
        let t = &self.theme;
        let label_x = 4.0;
        let swatch_x = 50.0;
        let mut y = 4.0;
        let row_height = 6.0;
        let group_gap = 3.0;

        // Header
        state.draw_screen(
            Text::new(&MONO_4X6, "darker dark  base  light lighter")
                .position(swatch_x, y)
                .color(Color::linear(0.5, 0.5, 0.5, 1.0)),
        );
        y += 8.0;

        let families: &[(&str, [(&str, &ColorScale); 3])] = &[
            ("Red", [("Pink", &t.pink), ("Red", &t.red), ("Maroon", &t.maroon)]),
            ("Orange", [("Peach", &t.peach), ("Orange", &t.orange), ("Rust", &t.rust)]),
            ("Yellow", [("Cream", &t.cream), ("Yellow", &t.yellow), ("Gold", &t.gold)]),
            ("Green", [("Mint", &t.mint), ("Green", &t.green), ("Forest", &t.forest)]),
            ("Blue", [("Sky", &t.sky), ("Blue", &t.blue), ("Navy", &t.navy)]),
            ("Purple", [("Lavender", &t.lavender), ("Purple", &t.purple), ("Indigo", &t.indigo)]),
            ("Brown", [("Tan", &t.tan), ("Brown", &t.brown), ("Chocolate", &t.chocolate)]),
            ("Gray", [("Silver", &t.silver), ("Gray", &t.gray), ("Charcoal", &t.charcoal)]),
        ];

        for (_family, shades) in families {
            for (name, scale) in shades {
                state.draw_screen(
                    Text::new(&MONO_4X6, *name)
                        .position(label_x, y)
                        .color(Color::linear(0.7, 0.7, 0.7, 1.0)),
                );
                self.draw_scale(state, scale, swatch_x, y);
                y += row_height;
            }
            y += group_gap;
        }

        // White/Black
        state.draw_screen(
            Text::new(&MONO_4X6, "White")
                .position(label_x, y)
                .color(Color::linear(0.7, 0.7, 0.7, 1.0)),
        );
        self.draw_scale(state, &t.white, swatch_x, y);
        y += row_height;

        state.draw_screen(
            Text::new(&MONO_4X6, "Black")
                .position(label_x, y)
                .color(Color::linear(0.7, 0.7, 0.7, 1.0)),
        );
        self.draw_scale(state, &t.black, swatch_x, y);
        y += row_height + group_gap;

        // Alpha demo
        state.draw_screen(
            Text::new(&MONO_4X6, "Alpha")
                .position(label_x, y)
                .color(Color::linear(0.7, 0.7, 0.7, 1.0)),
        );
        for i in 0..5 {
            let alpha = (i as f32 + 1.0) / 5.0;
            let sx = swatch_x + i as f32 * 6.0;
            state.draw_screen(
                Rect::from_top_left(sx, y, 5, 5)
                    .fill()
                    .color(t.blue.base.alpha(alpha)),
            );
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state == KeyState::Pressed && event.key == KeyCode::Escape {
            state.quit = true;
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let config = Config::builder()
        .title("Color Palette")
        .width(1024)
        .height(900)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    tiles::run(ColorDemo::new(), config).unwrap();
}
