use tiles::{
    App, Color, Config, Drawable, Element, ElementState, KeyCode, KeyEvent, KeyState, MouseEvent,
    Rect, Shape, State, StrokePosition,
};

struct Button {
    rect: Rect,
}

impl Button {
    fn new(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            rect: Rect::from_top_left(x, y, w, h),
        }
    }
}

impl Element for Button {
    fn shape(&self) -> impl Shape {
        self.rect
    }

    fn draw(&self, state: ElementState) -> impl Drawable {
        let color = match state {
            ElementState::Default => Color::linear(0.3, 0.3, 0.4, 1.0),
            ElementState::Hovered => Color::linear(0.4, 0.4, 0.6, 1.0),
            ElementState::Pressed => Color::linear(0.6, 0.5, 0.2, 1.0),
            ElementState::Captured => Color::linear(0.4, 0.3, 0.2, 1.0),
        };
        self.rect.rounded(3).fill().color(color)
    }
}

struct RoundButton {
    rect: Rect,
}

impl RoundButton {
    fn new(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            rect: Rect::from_top_left(x, y, w, h),
        }
    }
}

impl Element for RoundButton {
    fn shape(&self) -> impl Shape {
        self.rect.rounded(6)
    }

    fn draw(&self, state: ElementState) -> impl Drawable {
        let (fill_color, stroke_color) = match state {
            ElementState::Default => (
                Color::linear(0.15, 0.15, 0.2, 1.0),
                Color::linear(0.4, 0.4, 0.5, 1.0),
            ),
            ElementState::Hovered => (
                Color::linear(0.2, 0.2, 0.3, 1.0),
                Color::linear(0.6, 0.6, 0.8, 1.0),
            ),
            ElementState::Pressed => (
                Color::linear(0.1, 0.3, 0.5, 1.0),
                Color::linear(0.3, 0.7, 1.0, 1.0),
            ),
            ElementState::Captured => (
                Color::linear(0.15, 0.2, 0.3, 1.0),
                Color::linear(0.3, 0.5, 0.7, 1.0),
            ),
        };
        (
            self.rect.rounded(6).fill().color(fill_color),
            self.rect
                .rounded(6)
                .stroke(1, StrokePosition::Inner)
                .color(stroke_color),
        )
    }
}

struct DragBox {
    anchor: Rect,
    offset: (f32, f32),
}

impl DragBox {
    fn new(x: f32, y: f32, w: u32, h: u32) -> Self {
        Self {
            anchor: Rect::from_top_left(x, y, w, h),
            offset: (0.0, 0.0),
        }
    }

    fn visual_rect(&self) -> Rect {
        Rect::from_top_left(
            self.anchor.x() + self.offset.0,
            self.anchor.y() + self.offset.1,
            self.anchor.width(),
            self.anchor.height(),
        )
    }
}

impl Element for DragBox {
    fn shape(&self) -> impl Shape {
        self.anchor
    }

    fn draw(&self, state: ElementState) -> impl Drawable {
        let color = match state {
            ElementState::Default => Color::linear(0.3, 0.3, 0.4, 1.0),
            ElementState::Hovered => Color::linear(0.4, 0.4, 0.6, 1.0),
            ElementState::Pressed | ElementState::Captured => Color::linear(0.6, 0.5, 0.2, 1.0),
        };
        self.visual_rect().rounded(3).fill().color(color)
    }
}

struct Demo {
    click_count: u32,
    drag_box: DragBox,
}

impl App for Demo {
    fn init(&mut self, state: &mut State) {
        state.set_viewport_background(0.05, 0.05, 0.08, 1.0);
        state.set_window_background(0.0, 0.0, 0.0, 1.0);
        state.set_ambient_illumination(1.0);
    }

    fn update(&mut self, _state: &mut State) {}

    fn pre_update(&mut self, state: &mut State) {
        let button = Button::new(20.0, 20.0, 40, 16);
        let hit = button.handle_screen(state);
        if hit.is_clicked() {
            self.click_count += 1;
        }

        let round_button = RoundButton::new(20.0, 50.0, 50, 20);
        let hit = round_button.handle_screen(state);
        if hit.is_clicked() {
            self.click_count = 0;
        }

        let hit = self.drag_box.handle_screen(state);
        if let Some(drag) = hit.is_dragging() {
            self.drag_box.offset.0 = drag.total_delta_screen.x;
            self.drag_box.offset.1 = drag.total_delta_screen.y;
        } else if let Some(drag) = hit.is_drag_end() {
            self.drag_box.anchor = self.drag_box.anchor.translate(drag.total_delta_screen);
            self.drag_box.offset.0 = 0.0;
            self.drag_box.offset.1 = 0.0;
        }
    }

    fn draw(&mut self, state: &mut State) {
        let indicator_color = if self.click_count > 0 {
            let intensity = (self.click_count as f32 * 0.1).min(1.0);
            Color::linear(intensity, 0.8 - intensity * 0.5, 0.2, 1.0)
        } else {
            Color::linear(0.2, 0.2, 0.2, 1.0)
        };

        for i in 0..self.click_count.min(10) {
            state.draw_screen(
                Rect::from_top_left(20.0 + i as f32 * 8.0, 1.0, 6, 6)
                    .rounded(2)
                    .fill()
                    .color(indicator_color),
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
        .title("Element")
        .width(1024)
        .height(768)
        .viewport(256.0, 256.0)
        .no_file()
        .build();

    let demo = Demo {
        click_count: 0,
        drag_box: DragBox::new(150.0, 20.0, 30, 30),
    };

    tiles::run(demo, config).unwrap();
}
