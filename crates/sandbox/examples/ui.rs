use core::f32;
use std::f32::consts::PI;

use glam::Vec2;
use tiles::{
    font::TINY5_4X5,
    ui::{app_widget, col, row, text, widget, widget_fn},
    App, Cell, Color, Config, KeyCode, KeyEvent, KeyState, MouseEvent, Node, State, Text,
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
    pos: glam::Vec2,
}

#[widget_fn(Demo)]
fn button(
    word: impl Into<String>,
    f: impl Fn(&mut Demo, &mut State),
    children: Vec<Node<Demo>>,
) -> Node<Demo> {
    widget! {
        col()
        .align_center()
        .width(25)
        .color(BTN_COLOR)
        .hover_color(BTN_HOVER)
        .pressed_color(BTN_PRESS)
        .on_press(f) {
            text(word).font(&TINY5_4X5).padding(1)
            @children
        }
    }
}

#[widget_fn(Demo)]
fn border(c: Color, children: Vec<Node<Demo>>) -> Node<Demo> {
    widget! {
        row().gap(1).padding(5).color(c) {
            @children
        }
    }
}

#[widget_fn(Demo)]
fn action_bar(
    active_index: Option<usize>,
    set_active_index: impl Fn(&mut Demo, &mut State, usize) + Copy,
    actions: Vec<String>,
    children: Vec<Node<Demo>>,
) -> Node<Demo> {
    widget! {
        row().gap(1).padding(5) {
            @ for (i, child) in children.into_iter().take(actions.len()).enumerate() {
                col()
                .align_center()
                .width(25)
                .color(BTN_COLOR)
                .hover_color(BTN_HOVER)
                .pressed_color(BTN_PRESS)
                .on_press(move |app, state| {
                    set_active_index(app, state, i)
                }) {
                    text(&actions[i]).font(&TINY5_4X5).padding(1)
                    // @ if let Some(index) = active_index && index == i {

                    // }
                    col().relative(0.0, 0.0) {
                        child
                    }
                }
            }
            // @ for (i, child) in children.into_iter().enumerate() {
            //     if let Some(i) = &hovered_item_index {
            //         child
            //     }
            // }
        }
    }
}

#[app_widget]
impl App for Demo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.0, 0.0, 0.0, 1.0));
        state.set_ambient_illumination(1.0);
        state.set_debug(false);
    }

    fn ui(&self, _state: &State) -> Node<Self> {
        let _row_count = 20;
        let _pos = self.pos;
        let _elapsed = _state.elapsed();
        widget! {
            col().fill_w().fill_h() {
                // title bar
                action_bar(Some(0), |app, state, i| {}, vec![])
                row().padding(2).gap(2).fill_w() {
                    button("file", |app, _state| { app.count += 1; })
                    button("edit", |app, _state| { app.count -= 1; })
                    button("clear", |app, _state| app.count = 0)
                    button("debug", |_app, state| state.set_debug(!state.is_debug()))

                }
                col().fill_w().fill_h() {
                    // Counter indicator
                    // col().gap(1) {
                    //     @ for j in 0..=(self.count.unsigned_abs().saturating_sub(1) / row_count) {
                    //         row().gap(1) {
                    //             @ for i in 0..row_count {
                    //                 @ if j * row_count + i < self.count.unsigned_abs() {
                    //                     pane().size(4, 8).color(
                    //                         if self.count > 0 {
                    //                             INDICATOR
                    //                         } else {
                    //                             RED
                    //                         }
                    //                     );
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }

                    // Toggle pane
                    // pane()
                    //     .size(30, 10)
                    //     .color(if self.show_panel { BLUE } else { BTN_COLOR })
                    //     .hover_color(BTN_HOVER)
                    //     .pressed_color(BTN_PRESS)
                    //     .on_click(|app, _state| { app.show_panel = !app.show_panel; });

                    // Conditional panel
                    // @ if self.show_panel {
                    //     col().padding(3).gap(2).color(PANEL_BG) {
                    //         @ for (i, color) in self.items.iter().enumerate() {
                    //             pane()
                    //                 .size(40, 8)
                    //                 .color(*color)
                    //                 .hover_color(BTN_HOVER)
                    //                 .pressed_color(BTN_PRESS)
                    //                 .on_click(move |app, _state| {
                    //                     app.items.remove(i);
                    //                 });
                    //         }
                    //         pane()
                    //             .size(40, 8)
                    //             .color(BTN_COLOR)
                    //             .hover_color(BTN_HOVER)
                    //             .pressed_color(BTN_PRESS)
                    //             .on_click(|app, _state| {
                    //                 let c = match app.items.len() % 3 {
                    //                     0 => RED,
                    //                     1 => BLUE,
                    //                     _ => INDICATOR,
                    //                 };
                    //                 app.items.push(c);
                    //             });
                    //     }
                    // }
                }

                // Draggable pane
                // pane()
                //     .id("yo")
                //     .absolute(pos.x, pos.y)
                //     .size(60, 40)
                //     .color(BG)
                //     .hover_color(BTN_HOVER)
                //     .pressed_color(BTN_PRESS)
                //     .on_drag(|app, _state, drag| app.pos += drag.delta_screen) {
                //         text("drag me").font(&TINY5_4X5).padding(2)
                //     }
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
        state.debug_line(Vec2::ZERO, text_pos, Color::hex(0xFFFFFF));
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
        .viewport(256, 256)
        .no_file()
        .build();

    let demo = Demo {
        count: 0,
        items: vec![RED, BLUE, INDICATOR],
        show_panel: true,
        pos: Vec2::new(0.0, 0.0),
    };

    tiles::run(demo, config).unwrap();
}
