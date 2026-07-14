use tiles::{App, Color, Config, Image, State};

struct ImageDemo {
    logo: Option<Image>,
}

impl ImageDemo {
    fn new() -> Self {
        Self { logo: None }
    }
}

impl App for ImageDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.05, 0.05, 0.08, 1.0));

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/knight.png");
        self.logo = Some(Image::from_path(path).expect("failed to load sample image"));
    }

    fn draw(&mut self, state: &mut State) {
        if let Some(logo) = &self.logo {
            state.draw_world(logo.clone().position(0.0, 0.0).center());
        }
    }
}

fn main() {
    let config = Config::builder().title("Image").viewport(256, 256).build();
    tiles::run(ImageDemo::new(), config).unwrap();
}
