use tiles::{
    App, Color, Config, Node, State,
    ui::{app_widget, col, img, row, widget},
};

struct ImageWidgetDemo;

#[app_widget]
impl App for ImageWidgetDemo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.05, 0.05, 0.08, 1.0));

        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/knight.png");
        state
            .load_image("knight", path)
            .expect("failed to load sample image");
        state.set_debug(true);
    }

    fn ui(&self, _state: &State) -> Node<Self> {
        widget! {
            row().fill_w().fill_h().gap(8).padding(8) {
                col().gap(2) {
                    img("knight")
                }
                col().gap(2) {
                    img("missing")
                }
                col().gap(2) {
                    img("knight").size(40, 40)
                }
            }
        }
    }
}

fn main() {
    let config = Config::builder()
        .title("Image Widget")
        .viewport(256, 256)
        .build();
    tiles::run(ImageWidgetDemo, config).unwrap();
}
