use tiles::{
    App, Color, Config, Drawable, KeyCode, KeyEvent, KeyState, Line, MouseEvent, Rect, Shape,
    State, StrokePosition,
};

struct Shapes;

impl App for Shapes {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.0, 0.0, 0.0, 1.0));
        state.set_ambient_illumination(1.0);
    }

    fn update(&mut self, _state: &mut State) {}

    fn draw(&mut self, state: &mut State) {
        // Filled rect
        state.draw_world(
            Rect::from_top_left(10.0, 10.0, 30, 20)
                .fill()
                .color(Color::linear(0.8, 0.2, 0.2, 1.0)),
        );

        // Stroked rect (inner)
        state.draw_world(
            Rect::from_top_left(50.0, 10.0, 30, 20)
                .stroke(2, StrokePosition::Inner)
                .color(Color::linear(0.2, 0.8, 0.2, 1.0)),
        );

        // Stroked rect (outer)
        state.draw_world(
            Rect::from_top_left(92.0, 12.0, 26, 16)
                .stroke(2, StrokePosition::Outer)
                .color(Color::linear(0.2, 0.2, 0.8, 1.0)),
        );

        // Filled rounded rect
        state.draw_world(
            Rect::from_top_left(10.0, 50.0, 30, 20)
                .rounded(5)
                .fill()
                .color(Color::linear(0.8, 0.6, 0.1, 1.0)),
        );

        // Stroked rounded rect (inner)
        state.draw_world(
            Rect::from_top_left(50.0, 50.0, 30, 20)
                .rounded(5)
                .stroke(2, StrokePosition::Inner)
                .color(Color::linear(0.6, 0.1, 0.8, 1.0)),
        );

        // Rounded rect with different corner radii
        state.draw_world(
            Rect::from_top_left(90.0, 50.0, 30, 20)
                .rounded(0)
                .top_left(8)
                .bottom_right(8)
                .fill()
                .color(Color::linear(0.1, 0.7, 0.7, 1.0)),
        );

        // Lines at different widths
        state
            .draw_world(Line::new(10.0, 90.0, 50.0, 90.0).color(Color::linear(1.0, 1.0, 1.0, 1.0)));
        state.draw_world(
            Line::new(10.0, 95.0, 50.0, 95.0)
                .width(2)
                .color(Color::linear(0.8, 0.8, 0.2, 1.0)),
        );
        state.draw_world(
            Line::new(10.0, 102.0, 50.0, 102.0)
                .width(4)
                .color(Color::linear(0.2, 0.8, 0.8, 1.0)),
        );

        // Diagonal line
        state.draw_world(
            Line::new(60.0, 85.0, 100.0, 110.0)
                .width(2)
                .color(Color::linear(0.8, 0.4, 0.8, 1.0)),
        );

        // Filled + stroked (two draw calls)
        state.draw_world(
            Rect::from_top_left(140.0, 10.0, 40, 30)
                .rounded(6)
                .fill()
                .color(Color::linear(0.3, 0.3, 0.5, 1.0)),
        );
        state.draw_world(
            Rect::from_top_left(140.0, 10.0, 40, 30)
                .rounded(6)
                .stroke(1, StrokePosition::Inner)
                .color(Color::linear(1.0, 1.0, 1.0, 1.0)),
        );

        // Middle stroke
        state.draw_world(
            Rect::from_top_left(140.0, 55.0, 40, 30)
                .stroke(3, StrokePosition::Middle)
                .color(Color::linear(0.9, 0.5, 0.1, 1.0)),
        );

        // Expanded rect
        state.draw_world(
            Rect::from_top_left(140.0, 100.0, 20, 20)
                .expand(5)
                .fill()
                .color(Color::linear(0.4, 0.7, 0.3, 1.0)),
        );
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state == KeyState::Pressed && event.key == KeyCode::Escape {
            state.quit();
        }
    }

    fn on_mouse(&mut self, _state: &mut State, _event: MouseEvent) {}
}

fn main() {
    let config = Config::builder()
        .title("Shapes")
        .width(1024)
        .height(768)
        .viewport(256, 256)
        .no_file()
        .build();

    tiles::run(Shapes, config).unwrap();
}
