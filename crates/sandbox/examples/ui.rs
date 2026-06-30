use core::f32;
use std::f32::consts::PI;

use tiles::{
    font::TINY5_4X5,
    ui::{col, pane, row, text, Node},
    view, App, Cell, Color, Config, Drawable, KeyCode, KeyEvent, KeyState, MouseEvent, State, Text,
};

const BG: Color = Color::linear(0.12, 0.12, 0.15, 1.0);
const PANEL_BG: Color = Color::linear(0.50, 0.40, 0.22, 1.0);
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
        let row_count = 20;
        view! {
            col().padding(16).gap(4).fill_w().fill_h() {
                // Title bar
                row().fill_w().padding(1).gap(1).color(PANEL_BG) {
                    pane()
                        .align_center()
                        .fill_w()
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .text_color(INDICATOR)
                        .on_press(|app: &mut Demo, _state| { app.count += 1; })
                        .on_hold(|app: &mut Demo, _state| { app.count += 1; }) {
                            text("+").font(&TINY5_4X5).padding(1)
                    }
                    pane()
                        .align_center()
                        .fill_w()
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .text_color(RED)
                        .on_press(|app: &mut Demo, _state| { app.count -= 1; })
                        .on_hold(|app: &mut Demo, _state| { app.count -= 1; }) {
                            text("-").font(&TINY5_4X5).padding(1)
                    }
                    pane()
                        .align_center()
                        .fill_w()
                        .color(BTN_COLOR)
                        .hover_color(BTN_HOVER)
                        .pressed_color(BTN_PRESS)
                        .on_click(|app: &mut Demo, _state| { app.count = 0; }) {
                            text("clear").font(&TINY5_4X5).padding(1)
                    }
                }

                // Counter indicator
                col().gap(1) {
                    @ for j in 0..=(self.count.unsigned_abs().saturating_sub(1) / row_count) {
                        row().gap(1) {
                            @ for i in 0..row_count {
                                @ if j * row_count + i < self.count.unsigned_abs() {
                                    pane().size(4, 8).color(
                                        if self.count > 0 {
                                            INDICATOR
                                        } else {
                                            RED
                                        }
                                    );
                                }
                            }
                        }
                    }
                }

                // Toggle pane
                pane()
                    .size(30, 10)
                    .color(if self.show_panel { BLUE } else { BTN_COLOR })
                    .hover_color(BTN_HOVER)
                    .pressed_color(BTN_PRESS)
                    .on_click(|app: &mut Demo, _state| { app.show_panel = !app.show_panel; });

                // Conditional panel
                @ if self.show_panel {
                    col().padding(3).gap(2).color(PANEL_BG) {
                        @ for (i, color) in self.items.iter().enumerate() {
                            pane()
                                .size(40, 8)
                                .color(*color)
                                .hover_color(BTN_HOVER)
                                .pressed_color(BTN_PRESS)
                                .on_click(move |app: &mut Demo, _state| {
                                    app.items.remove(i);
                                });
                        }
                        pane()
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
            }
        }
    }

    fn on_key(&mut self, state: &mut State, event: KeyEvent) {
        if event.state == KeyState::Pressed && event.key == KeyCode::Escape {
            state.quit();
        }
    }

    fn draw(&mut self, state: &mut State) {
        let elapsed = state.elapsed();
        let text_pos = glam::Vec2::from_angle(elapsed * PI / 2.0) * 30.0;
        state.draw_world(
            Text::new(&TINY5_4X5, "Hello World")
                .center()
                .map_position(move |i, _c| {
                    (0.0, 0.5 * f32::sin((elapsed + i as f32 * 10.0) * (10.0)))
                })
                .position(text_pos.x, text_pos.y),
        );
        state.draw_world(Cell::new(0.0, 0.0).color(Color::rgb8(0, 0, 0)));
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
