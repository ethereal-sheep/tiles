use tiles::{
    ui,
    ui::{button, col, row, Node},
    App, Color, Config, KeyCode, KeyEvent, KeyState, MouseEvent, State,
};

const BG: Color = Color::linear(0.12, 0.12, 0.15, 1.0);
const PANEL_BG: Color = Color::linear(0.18, 0.18, 0.22, 1.0);
const BTN_COLOR: Color = Color::linear(0.25, 0.25, 0.35, 1.0);
const BTN_HOVER: Color = Color::linear(0.35, 0.35, 0.50, 1.0);
const BTN_PRESS: Color = Color::linear(0.50, 0.40, 0.20, 1.0);
const INDICATOR: Color = Color::linear(0.2, 0.8, 0.4, 1.0);
const RED: Color = Color::linear(0.8, 0.2, 0.2, 1.0);
const BLUE: Color = Color::linear(0.2, 0.4, 0.9, 1.0);

struct Demo {
    count: i32,
    items: Vec<Color>,
    show_panel: bool,
}

impl App for Demo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.05, 0.05, 0.08, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(1.0);
    }

    fn ui(&self, _state: &State) -> Node<Self> {
        col()
            .padding(16)
            .gap(4)
            .color(BG)
            .fill_w()
            .fill_h()
            .children(ui! {
                // Title bar
                row().gap(4).padding(2).color(PANEL_BG) {
                    button()
                        .size(20, 10)
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .on_click(|app: &mut Demo, _state| { app.count += 1; });
                    button()
                        .size(20, 10)
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .on_click(|app: &mut Demo, _state| { app.count -= 1; });
                    button()
                        .size(20, 10)
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .on_click(|app: &mut Demo, _state| { app.count = 0; });
                }

                // Counter indicator
                row().gap(1) {
                    @ for _i in 0..self.count.unsigned_abs().min(20) {
                        Node::new().size(4, 8).color(
                            if self.count > 0 { INDICATOR } else { RED }
                        );
                    }
                }

                // Toggle button
                button()
                    .size(30, 10)
                    .color(if self.show_panel { BLUE } else { BTN_COLOR })
                    .hover_color(BTN_HOVER)
                    .pressed_color(BTN_PRESS)
                    .on_click(|app: &mut Demo, _state| { app.show_panel = !app.show_panel; });

                // Conditional panel
                @ if self.show_panel {
                    col().padding(3).gap(2).color(PANEL_BG) {
                        @ for (i, color) in self.items.iter().enumerate() {
                            button()
                                .size(40, 8)
                                .color(*color)
                                .hover_color(BTN_HOVER)
                                .pressed_color(BTN_PRESS)
                                .on_click(move |app: &mut Demo, _state| {
                                    app.items.remove(i);
                                });
                        }
                        button()
                            .size(40, 8)
                            .color(BTN_COLOR)
                            .hover_color(BTN_HOVER)
                            .pressed_color(BTN_PRESS)
                            .on_click(|app: &mut Demo, _state| {
                                let c = match app.items.len() % 3 {
                                    0 => RED,
                                    1 => BLUE,
                                    _ => INDICATOR,
                                };
                                app.items.push(c);
                            });
                    }
                }
            })
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
        .title("UI Demo")
        .width(1024)
        .height(768)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    let demo = Demo {
        count: 0,
        items: vec![RED, BLUE, INDICATOR],
        show_panel: true,
    };

    tiles::run(demo, config).unwrap();
}
