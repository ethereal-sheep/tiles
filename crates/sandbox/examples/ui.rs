use core::f32;
use std::f32::consts::PI;

use glam::Vec2;
use tiles::{
    App, Cell, Color, Config, Drawable, Handlers, KeyCode, KeyEvent, KeyState, MouseEvent, Node,
    Props, Rect, Shape, State, Style, Text,
    font::TINY5_4X5,
    ui::{
        col, get_app, get_state, img,
        macros::{widget, widget_fn},
        paint, row, signal, text,
    },
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
    active_index: Option<usize>,
}

#[widget_fn]
fn new_widget_with_props(
    name: &str,
    Props {
        style,
        handlers,
        children,
    }: Props,
) -> Node {
    widget! {
        col().style(style).handlers(handlers)
            .width(25)
            .color(BTN_COLOR)
            .hover_color(BTN_HOVER)
            .pressed_color(BTN_PRESS) {
            text(name).font(&TINY5_4X5).padding(1)
            @children
        }
    }
}

#[widget_fn]
fn new_widget_with_explicit_merged_style_and_props(
    name: &str,
    Props {
        style,
        handlers,
        children,
    }: Props,
) -> Node {
    let new_style = Style {
        gap: style.gap + 1,
        ..style
    };

    widget! {
        col().style(new_style).handlers(handlers) {
            text(name).font(&TINY5_4X5).padding(1)
            @children
        }
    }
}

#[widget_fn] // generate a type which doesn't pass in node data, since user never ask for it
fn new_widget_without_props(name: &str) -> Node {
    widget! {
        col().on_click(|| get_app::<Demo>().with_mut(|app| app.show_panel = true)) {
            text(name).font(&TINY5_4X5).padding(1)
        }
    }
}

#[widget_fn]
fn test(
    name: &str,
    Props {
        style,
        handlers,
        children,
    }: Props,
) -> Node {
    widget! {
        col().style(style).handlers(handlers) {
            text(name).font(&TINY5_4X5).padding(1)
            @children
        }
    }
}

#[widget_fn]
fn button(
    word: impl Into<String>,
    Props {
        handlers, children, ..
    }: Props,
) -> Node {
    let handlers = Handlers {
        on_press: handlers.on_press,
        ..Default::default()
    };

    widget! {
        col()
        .align_center()
        .width(25)
        .color(BTN_COLOR)
        .hover_color(BTN_HOVER)
        .pressed_color(BTN_PRESS)
        .handlers(handlers) {
            text(word).font(&TINY5_4X5).padding(1)
            @children
        }
    }
}

#[widget_fn]
fn border(c: Color, Props { children, .. }: Props) -> Node {
    widget! {
        row().gap(1).padding(5).color(c) {
            @children
        }
    }
}

#[widget_fn]
fn signal_counter() -> Node {
    let count = signal(0i32);
    widget! {
        row().gap(2).padding(2) {
            text(format!("{}", count.get())).width(25).justify_center().font(&TINY5_4X5).padding(1).color(BTN_PRESS)
            // Copy handler handle — reused across two buttons
            col().width(25).color(BTN_COLOR).hover_color(BTN_HOVER).pressed_color(BTN_PRESS)
                .on_press(move || {
                    count.set(count.get() + 1);
                }) {
                text("inc").font(&TINY5_4X5).padding(1)
            }
            col().width(25).color(BTN_COLOR).hover_color(BTN_HOVER).pressed_color(BTN_PRESS)
                .on_press(move || {
                    get_app::<Demo>().with_mut(|app| app.count += 1);
                    count.set(count.get() - 1);
                }) {
                text("dec").font(&TINY5_4X5).padding(1)
            }
        }
    }
}

// #[widget_fn]
// fn action_bar(
//     active_index: Option<usize>,
//     set_active_index: impl Fn(usize) + Copy,
//     actions: Vec<&str>,
//     children: Vec<Node>,
// ) -> Node {
//     widget! {
//         row().gap(1).padding(5) {
//             @ for (i, child) in children.into_iter().take(actions.len()).enumerate() {
//                 col()
//                 .align_center()
//                 .width(25)
//                 .color(BTN_COLOR)
//                 .hover_color(BTN_HOVER)
//                 .pressed_color(BTN_PRESS)
//                 .on_press(move || {
//                     set_active_index(i)
//                 }) {
//                     text(actions[i]).font(&TINY5_4X5).padding(1)
//                     @ if let Some(index) = active_index && index == i {
//                         col().relative(5.0, 0.0) {
//                             @[child]
//                             text("testing hide")
//                         }
//                     }
//                 }
//             }
//             // @ for (i, child) in children.into_iter().enumerate() {
//             //     if let Some(i) = &hovered_item_index {
//             //         child
//             //     }
//             // }
//         }
//     }
// }

impl App for Demo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(Color::linear(0.05, 0.05, 0.08, 1.0));
        state.set_window_background(Color::linear(0.0, 0.0, 0.0, 1.0));
        state.set_ambient_illumination(1.0);
        state.set_debug(false);
    }

    fn ui() -> Node {
        let (fps, _elapsed, _pos, active_index) = get_app::<Demo>().with(|app| {
            get_state().with(|state| (1.0 / state.dt(), state.elapsed(), app.pos, app.active_index))
        });
        let _row_count = 20;
        let app = get_app::<Self>();
        widget! {
            col().fill_w().fill_h() {
                row().padding(2).gap(2).fill_w() {
                    button("file")
                    button("edit")
                    button("clear")
                    button("debug").on_press(move || get_state().with_mut(|state| state.set_debug(!state.is_debug())))
                }
                // title bar
                // action_bar(active_index, move |i| { app.with_mut(|app| app.active_index = Some(i)); eprintln!("{}", i); }, vec!["test1"]) {
                //     button("file", move || app.with_mut(|app| app.count += 1))
                // }
                col().fill_w().fill_h() {
                    @ for i in 0..3 {
                        signal_counter()
                    }
                    signal_counter()
                    img("knight")
                    paint(Rect::from_top_left(0.0, 0.0, 12, 12).fill().color(RED))
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
                    //     .on_click(|| { app.with_mut(|app| app.show_panel = !app.show_panel); });

                    // Conditional panel
                    // @ if self.show_panel {
                    //     col().padding(3).gap(2).color(PANEL_BG) {
                    //         @ for (i, color) in self.items.iter().enumerate() {
                    //             pane()
                    //                 .size(40, 8)
                    //                 .color(*color)
                    //                 .hover_color(BTN_HOVER)
                    //                 .pressed_color(BTN_PRESS)
                    //                 .on_click(move || {
                    //                     app.with_mut(|app| { app.items.remove(i); });
                    //                 });
                    //         }
                    //         pane()
                    //             .size(40, 8)
                    //             .color(BTN_COLOR)
                    //             .hover_color(BTN_HOVER)
                    //             .pressed_color(BTN_PRESS)
                    //             .on_click(|| {
                    //                 app.with_mut(|app| {
                    //                     let c = match app.items.len() % 3 {
                    //                         0 => RED,
                    //                         1 => BLUE,
                    //                         _ => INDICATOR,
                    //                     };
                    //                     app.items.push(c);
                    //                 });
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
                //     .on_drag(|drag| app.with_mut(|app| app.pos += drag.delta_screen)) {
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
        active_index: None,
    };

    tiles::run(demo, config).unwrap();
}
