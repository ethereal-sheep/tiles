use tiles::{App, Color, Config, Image, KeyCode::Space, Sprite, State};

struct SpriteDemo {
    bounce: Option<Sprite>,
    cycle: Option<Sprite>,
    elapsed: f32,
    paused: bool,
}

impl SpriteDemo {
    fn new() -> Self {
        Self {
            bounce: None,
            cycle: None,
            elapsed: 0.0,
            paused: false,
        }
    }
}

impl App for SpriteDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.05, 0.05, 0.08, 1.0));

        let sheet_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/bounce_sheet.png");
        let sheet = Image::from_path(sheet_path).expect("failed to load bounce_sheet.png");
        self.bounce = Some(Sprite::new(&sheet).grid(8, 1).repeat());

        let gif_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/color_cycle.gif");
        let gif = Image::from_path(gif_path).expect("failed to load color_cycle.gif");
        self.cycle = Some(Sprite::new(&gif).repeat());
    }

    fn draw(&mut self, state: &mut State) {
        if !self.paused {
            self.elapsed += state.dt();
        }
        if let Some(bounce) = &self.bounce {
            state.draw_world(
                bounce
                    .frame_at(self.elapsed)
                    .position(-40.0, 0.0)
                    .center_left(),
            );
        }
        if let Some(cycle) = &self.cycle {
            state.draw_world(cycle.frame_at(self.elapsed).position(20.0, 0.0).center());
        }
    }

    fn on_key(&mut self, _state: &mut State, event: tiles::KeyEvent) {
        if event.key == Space && event.state == tiles::KeyState::Pressed {
            self.paused = !self.paused;
        }
    }
}

fn main() {
    let config = Config::builder().title("Sprite").viewport(256, 256).build();
    tiles::run(SpriteDemo::new(), config).unwrap();
}
