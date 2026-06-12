use glam::Vec2;
use std::collections::{HashMap, HashSet};

use crate::rect::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Space,
    Enter,
    Escape,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub state: KeyState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseAction {
    Pressed(MouseButton),
    Released(MouseButton),
    Moved {
        screen_delta: Vec2,
        world_delta: Vec2,
    },
    Scrolled(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub screen_pos: Vec2,
    pub world_pos: Vec2,
    pub viewport_pos: Vec2,
}

pub(crate) struct InputState {
    pub keys_down: HashSet<KeyCode>,
    pub mouse_buttons_down: HashSet<MouseButton>,
    pub mouse_screen_pos: Vec2,
    pub mouse_world_pos: Vec2,
    pub prev_mouse_screen_pos: Vec2,
    pub prev_mouse_world_pos: Vec2,
    pub left_press_screen_pos: Vec2,
    pub left_press_world_pos: Vec2,
    pub scroll_delta: f32,
    pub keys_states: HashMap<KeyCode, ButtonState>,
    pub mouse_buttons_states: HashMap<MouseButton, ButtonState>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keys_down: HashSet::new(),
            mouse_buttons_down: HashSet::new(),
            mouse_screen_pos: Vec2::ZERO,
            mouse_world_pos: Vec2::ZERO,
            prev_mouse_screen_pos: Vec2::ZERO,
            prev_mouse_world_pos: Vec2::ZERO,
            left_press_screen_pos: Vec2::ZERO,
            left_press_world_pos: Vec2::ZERO,
            scroll_delta: 0.0,
            keys_states: HashMap::new(),
            mouse_buttons_states: HashMap::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.scroll_delta = 0.0;
    }

    pub fn update(&mut self, dt: f32, elapsed: f32) {
        for (_, state) in self.keys_states.iter_mut() {
            state.update(dt, elapsed);
        }
        for (_, state) in self.mouse_buttons_states.iter_mut() {
            state.update(dt, elapsed);
        }
    }

    pub fn reset(&mut self) {
        for (_, state) in self.keys_states.iter_mut() {
            state.reset();
        }
        for (_, state) in self.mouse_buttons_states.iter_mut() {
            state.reset();
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_down())
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_pressed())
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_released())
    }

    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_clicked())
    }

    pub fn is_key_double_clicked(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_double_clicked())
    }

    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_held())
    }

    pub fn is_key_released_after_hold(&self, key: KeyCode) -> bool {
        self.keys_states
            .get(&key)
            .is_some_and(|state| state.is_released_after_hold())
    }

    pub fn is_mouse_down(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_down())
    }

    pub fn is_mouse_pressed(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_pressed())
    }

    pub fn is_mouse_released(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_released())
    }

    pub fn is_mouse_clicked(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_clicked())
    }

    pub fn is_mouse_double_clicked(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_double_clicked())
    }

    pub fn is_mouse_held(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_held())
    }

    pub fn is_mouse_released_after_hold(&self, mouse: MouseButton) -> bool {
        self.mouse_buttons_states
            .get(&mouse)
            .is_some_and(|state| state.is_released_after_hold())
    }

    pub fn test_rect_world(&self, rect: &Rect) -> RectInputState {
        self.test_rect_impl(
            rect,
            self.mouse_world_pos,
            self.prev_mouse_world_pos,
            self.left_press_world_pos,
        )
    }

    pub fn test_rect_screen(&self, rect: &Rect) -> RectInputState {
        self.test_rect_impl(
            rect,
            self.mouse_screen_pos,
            self.prev_mouse_screen_pos,
            self.left_press_screen_pos,
        )
    }

    fn test_rect_impl(
        &self,
        rect: &Rect,
        mouse_pos: Vec2,
        prev_mouse_pos: Vec2,
        press_pos: Vec2,
    ) -> RectInputState {
        let hovered = rect.contains_point(mouse_pos.x, mouse_pos.y);
        let was_hovered = rect.contains_point(prev_mouse_pos.x, prev_mouse_pos.y);
        let just_entered = hovered && !was_hovered;
        let just_left = !hovered && was_hovered;

        let left = self.mouse_buttons_states.get(&MouseButton::Left);
        let left_down = left.is_some_and(|s| s.is_down());
        let left_pressed_this_frame = left.is_some_and(|s| s.pressed_this_frame);
        let left_released_this_frame = left.is_some_and(|s| s.released_this_frame);
        let left_held_duration = left.map_or(0.0, |s| s.held_duration);

        let press_was_inside = rect.contains_point(press_pos.x, press_pos.y);

        let pressed = hovered && left_pressed_this_frame;
        let released = hovered && left_released_this_frame && press_was_inside;
        let clicked = released && left_held_duration < HOLD_THRESHOLD_SECS;
        let double_clicked = clicked && left.is_some_and(|s| s.press_count >= 2);

        let held = left_down && press_was_inside && left_held_duration >= HOLD_THRESHOLD_SECS;
        let hold_duration = if press_was_inside { left_held_duration } else { 0.0 };
        let released_after_hold = released && left_held_duration >= HOLD_THRESHOLD_SECS;

        let right = self.mouse_buttons_states.get(&MouseButton::Right);
        let right_clicked = hovered
            && right.is_some_and(|s| s.released_this_frame && s.held_duration < HOLD_THRESHOLD_SECS);

        let middle = self.mouse_buttons_states.get(&MouseButton::Middle);
        let middle_clicked = hovered
            && middle.is_some_and(|s| s.released_this_frame && s.held_duration < HOLD_THRESHOLD_SECS);

        let drag_delta_screen = if left_down && press_was_inside {
            self.mouse_screen_pos - self.prev_mouse_screen_pos
        } else {
            Vec2::ZERO
        };
        let drag_delta_world = if left_down && press_was_inside {
            self.mouse_world_pos - self.prev_mouse_world_pos
        } else {
            Vec2::ZERO
        };
        let drag_origin_screen = self.left_press_screen_pos;
        let drag_origin_world = self.left_press_world_pos;
        let is_dragging = left_down
            && press_was_inside
            && (self.mouse_screen_pos - self.left_press_screen_pos).length() >= DRAG_MIN_PIXELS;

        let scroll_delta = if hovered { self.scroll_delta } else { 0.0 };

        RectInputState {
            hovered,
            just_entered,
            just_left,
            pressed,
            released,
            clicked,
            double_clicked,
            held,
            hold_duration,
            released_after_hold,
            right_clicked,
            middle_clicked,
            drag_delta_screen,
            drag_delta_world,
            drag_origin_screen,
            drag_origin_world,
            is_dragging,
            scroll_delta,
        }
    }
}

pub const HOLD_THRESHOLD_SECS: f32 = 0.4;
pub const DOUBLE_CLICK_THRESHOLD_SECS: f32 = 0.25;
pub const DRAG_MIN_PIXELS: f32 = 4.0;

#[derive(Debug, Clone, Copy)]
pub struct RectInputState {
    pub hovered: bool,
    pub just_entered: bool,
    pub just_left: bool,

    pub pressed: bool,
    pub released: bool,
    pub clicked: bool,
    pub double_clicked: bool,
    pub held: bool,
    pub hold_duration: f32,
    pub released_after_hold: bool,

    pub right_clicked: bool,
    pub middle_clicked: bool,

    pub drag_delta_screen: Vec2,
    pub drag_delta_world: Vec2,
    pub drag_origin_screen: Vec2,
    pub drag_origin_world: Vec2,
    pub is_dragging: bool,

    pub scroll_delta: f32,
}

#[derive(Debug, Clone)]
pub struct ButtonState {
    pub held_duration: f32,
    pub last_release_time: f32,
    pub press_count: u8,
    pub pressed_this_frame: bool,
    pub released_this_frame: bool,
    pub is_down: bool,
}

impl ButtonState {
    pub fn new() -> Self {
        Self {
            held_duration: 0.0,
            last_release_time: f32::NEG_INFINITY,
            press_count: 0,
            pressed_this_frame: false,
            released_this_frame: false,
            is_down: false,
        }
    }

    pub fn update(&mut self, dt: f32, elapsed: f32) {
        if self.pressed_this_frame {
            self.held_duration = 0.0;
        }

        if self.is_down {
            self.held_duration += dt;
        }

        if self.released_this_frame {
            let since_last = elapsed - self.last_release_time;
            if since_last <= DOUBLE_CLICK_THRESHOLD_SECS {
                self.press_count = self.press_count.saturating_add(1);
            } else {
                self.press_count = 1;
            }
            self.last_release_time = elapsed;
        } else if !self.is_down {
            // Reset click streak if window has expired
            if elapsed - self.last_release_time > DOUBLE_CLICK_THRESHOLD_SECS {
                self.press_count = 0;
            }
        }
    }

    pub fn reset(&mut self) {
        self.pressed_this_frame = false;
        self.released_this_frame = false;
    }

    pub fn is_down(&self) -> bool {
        self.is_down
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed_this_frame
    }

    pub fn is_released(&self) -> bool {
        self.released_this_frame
    }

    pub fn is_clicked(&self) -> bool {
        self.is_released() && self.held_duration < HOLD_THRESHOLD_SECS
    }

    pub fn is_double_clicked(&self) -> bool {
        self.is_clicked() && self.press_count >= 2
    }

    pub fn is_held(&self) -> bool {
        self.held_duration >= HOLD_THRESHOLD_SECS
    }

    pub fn is_released_after_hold(&self) -> bool {
        self.is_released() && self.held_duration >= HOLD_THRESHOLD_SECS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_button() -> ButtonState {
        ButtonState::new()
    }

    fn press(state: &mut ButtonState) {
        state.pressed_this_frame = true;
        state.is_down = true;
    }

    fn release(state: &mut ButtonState) {
        state.released_this_frame = true;
        state.is_down = false;
    }

    #[test]
    fn new_button_state_is_idle() {
        let s = make_button();
        assert!(!s.is_down());
        assert!(!s.is_pressed());
        assert!(!s.is_released());
        assert!(!s.is_clicked());
        assert!(!s.is_double_clicked());
        assert!(!s.is_held());
        assert!(!s.is_released_after_hold());
        assert_eq!(s.press_count, 0);
    }

    #[test]
    fn press_and_quick_release_is_click() {
        let mut s = make_button();

        // Frame 1: press
        press(&mut s);
        s.update(0.016, 1.0);
        assert!(s.is_pressed());
        assert!(s.is_down());
        assert!(!s.is_clicked());

        s.reset();

        // Frame 2: release after short hold
        release(&mut s);
        s.update(0.016, 1.016);
        assert!(s.is_released());
        assert!(s.is_clicked());
        assert!(!s.is_held());
        assert!(!s.is_released_after_hold());
        assert_eq!(s.press_count, 1);
    }

    #[test]
    fn hold_past_threshold() {
        let mut s = make_button();

        press(&mut s);
        s.update(0.016, 1.0);
        s.reset();

        // Simulate frames until hold threshold is passed
        let mut elapsed = 1.016;
        for _ in 0..30 {
            s.update(0.016, elapsed);
            elapsed += 0.016;
        }

        assert!(s.is_down());
        assert!(s.is_held());
        assert!(s.held_duration >= HOLD_THRESHOLD_SECS);
    }

    #[test]
    fn release_after_hold() {
        let mut s = make_button();

        press(&mut s);
        s.update(0.016, 1.0);
        s.reset();

        // Hold for 0.5s
        let mut elapsed = 1.016;
        for _ in 0..31 {
            s.update(0.016, elapsed);
            elapsed += 0.016;
        }

        // Release
        release(&mut s);
        s.update(0.016, elapsed);

        assert!(s.is_released());
        assert!(s.is_released_after_hold());
        assert!(!s.is_clicked()); // not a click because it was held
    }

    #[test]
    fn double_click() {
        let mut s = make_button();
        let mut elapsed = 1.0;

        // First click
        press(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        release(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.05; // short gap

        // Second click
        press(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        release(&mut s);
        s.update(0.016, elapsed);

        assert!(s.is_clicked());
        assert!(s.is_double_clicked());
        assert_eq!(s.press_count, 2);
    }

    #[test]
    fn double_click_window_expires() {
        let mut s = make_button();
        let mut elapsed = 1.0;

        // First click
        press(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        release(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.5; // long gap — exceeds DOUBLE_CLICK_THRESHOLD_SECS

        // Let the streak expire
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        // Second click
        press(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        release(&mut s);
        s.update(0.016, elapsed);

        assert!(s.is_clicked());
        assert!(!s.is_double_clicked());
        assert_eq!(s.press_count, 1);
    }

    #[test]
    fn reset_clears_frame_flags() {
        let mut s = make_button();

        press(&mut s);
        s.update(0.016, 1.0);
        assert!(s.is_pressed());

        s.reset();
        assert!(!s.is_pressed());
        assert!(!s.is_released());
    }

    #[test]
    fn held_duration_resets_on_new_press() {
        let mut s = make_button();
        let mut elapsed = 1.0;

        // First press, hold a bit
        press(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.016;

        s.update(0.1, elapsed);
        elapsed += 0.1;
        assert!(s.held_duration > 0.0);

        // Release
        release(&mut s);
        s.update(0.016, elapsed);
        s.reset();
        elapsed += 0.5;

        // New press should reset held_duration
        press(&mut s);
        s.update(0.016, elapsed);
        assert_eq!(s.held_duration, 0.016);
    }

    #[test]
    fn input_state_key_queries() {
        let mut input = InputState::new();

        assert!(!input.is_key_down(KeyCode::A));
        assert!(!input.is_key_pressed(KeyCode::A));

        // Simulate pressing A
        let state = input.keys_states.entry(KeyCode::A).or_insert(ButtonState::new());
        state.pressed_this_frame = true;
        state.is_down = true;
        state.update(0.016, 1.0);

        assert!(input.is_key_down(KeyCode::A));
        assert!(input.is_key_pressed(KeyCode::A));
        assert!(!input.is_key_released(KeyCode::A));
    }

    #[test]
    fn input_state_mouse_queries() {
        let mut input = InputState::new();

        assert!(!input.is_mouse_down(MouseButton::Left));

        let state = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        state.pressed_this_frame = true;
        state.is_down = true;
        state.update(0.016, 1.0);

        assert!(input.is_mouse_down(MouseButton::Left));
        assert!(input.is_mouse_pressed(MouseButton::Left));
    }

    #[test]
    fn input_state_begin_frame_resets_scroll() {
        let mut input = InputState::new();
        input.scroll_delta = 3.0;
        input.begin_frame();
        assert_eq!(input.scroll_delta, 0.0);
    }

    #[test]
    fn input_state_update_propagates_to_button_states() {
        let mut input = InputState::new();

        let state = input.keys_states.entry(KeyCode::Space).or_insert(ButtonState::new());
        state.pressed_this_frame = true;
        state.is_down = true;

        input.update(0.016, 1.0);

        assert_eq!(input.keys_states[&KeyCode::Space].held_duration, 0.016);
    }

    #[test]
    fn input_state_reset_clears_all_frame_flags() {
        let mut input = InputState::new();

        let state = input.keys_states.entry(KeyCode::W).or_insert(ButtonState::new());
        state.pressed_this_frame = true;
        state.is_down = true;

        let mstate = input
            .mouse_buttons_states
            .entry(MouseButton::Right)
            .or_insert(ButtonState::new());
        mstate.released_this_frame = true;

        input.reset();

        assert!(!input.keys_states[&KeyCode::W].pressed_this_frame);
        assert!(!input.mouse_buttons_states[&MouseButton::Right].released_this_frame);
    }

    fn make_rect() -> Rect {
        Rect::from_top_left(10.0, 10.0, 20, 20)
    }

    fn input_with_mouse_at(screen: Vec2, world: Vec2) -> InputState {
        let mut input = InputState::new();
        input.mouse_screen_pos = screen;
        input.mouse_world_pos = world;
        input.prev_mouse_screen_pos = screen;
        input.prev_mouse_world_pos = world;
        input
    }

    #[test]
    fn test_rect_hovered_when_inside() {
        let input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.hovered);
        assert!(!state.just_entered);
        assert!(!state.just_left);
    }

    #[test]
    fn test_rect_not_hovered_when_outside() {
        let input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.hovered);
    }

    #[test]
    fn test_rect_just_entered() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        input.prev_mouse_screen_pos = Vec2::new(5.0, 5.0);
        input.prev_mouse_world_pos = Vec2::new(5.0, 5.0);
        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.hovered);
        assert!(state.just_entered);
        assert!(!state.just_left);
    }

    #[test]
    fn test_rect_just_left() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        input.prev_mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.prev_mouse_world_pos = Vec2::new(15.0, 15.0);
        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.hovered);
        assert!(!state.just_entered);
        assert!(state.just_left);
    }

    #[test]
    fn test_rect_pressed_inside() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.pressed_this_frame = true;
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.pressed);
        assert!(!state.released);
        assert!(!state.clicked);
    }

    #[test]
    fn test_rect_pressed_outside_not_registered() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.pressed_this_frame = true;
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(5.0, 5.0);
        input.left_press_world_pos = Vec2::new(5.0, 5.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.pressed);
    }

    #[test]
    fn test_rect_click_requires_press_inside() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.is_down = false;
        left.held_duration = 0.05;
        left.press_count = 1;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.released);
        assert!(state.clicked);
        assert!(!state.double_clicked);
    }

    #[test]
    fn test_rect_click_rejected_when_press_outside() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.is_down = false;
        left.held_duration = 0.05;
        input.left_press_screen_pos = Vec2::new(5.0, 5.0);
        input.left_press_world_pos = Vec2::new(5.0, 5.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.released);
        assert!(!state.clicked);
    }

    #[test]
    fn test_rect_double_click() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.is_down = false;
        left.held_duration = 0.05;
        left.press_count = 2;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.clicked);
        assert!(state.double_clicked);
    }

    #[test]
    fn test_rect_hold() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        left.held_duration = 0.5;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.held);
        assert_eq!(state.hold_duration, 0.5);
    }

    #[test]
    fn test_rect_released_after_hold() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.released_this_frame = true;
        left.is_down = false;
        left.held_duration = 0.5;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.released_after_hold);
        assert!(!state.clicked);
    }

    #[test]
    fn test_rect_right_click() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let right = input
            .mouse_buttons_states
            .entry(MouseButton::Right)
            .or_insert(ButtonState::new());
        right.released_this_frame = true;
        right.is_down = false;
        right.held_duration = 0.05;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(state.right_clicked);
        assert!(!state.middle_clicked);
    }

    #[test]
    fn test_rect_middle_click() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let middle = input
            .mouse_buttons_states
            .entry(MouseButton::Middle)
            .or_insert(ButtonState::new());
        middle.released_this_frame = true;
        middle.is_down = false;
        middle.held_duration = 0.05;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.right_clicked);
        assert!(state.middle_clicked);
    }

    #[test]
    fn test_rect_drag_delta() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(20.0, 20.0);
        input.mouse_world_pos = Vec2::new(20.0, 20.0);
        input.prev_mouse_screen_pos = Vec2::new(18.0, 18.0);
        input.prev_mouse_world_pos = Vec2::new(18.0, 18.0);
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        left.held_duration = 0.1;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert_eq!(state.drag_delta_screen, Vec2::new(2.0, 2.0));
        assert_eq!(state.drag_delta_world, Vec2::new(2.0, 2.0));
        assert!(state.is_dragging);
    }

    #[test]
    fn test_rect_drag_not_active_below_threshold() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(16.0, 15.0);
        input.mouse_world_pos = Vec2::new(16.0, 15.0);
        input.prev_mouse_screen_pos = Vec2::new(15.5, 15.0);
        input.prev_mouse_world_pos = Vec2::new(15.5, 15.0);
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        left.held_duration = 0.05;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.is_dragging);
        assert_ne!(state.drag_delta_screen, Vec2::ZERO);
    }

    #[test]
    fn test_rect_drag_not_active_when_press_outside() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(20.0, 20.0);
        input.mouse_world_pos = Vec2::new(20.0, 20.0);
        input.prev_mouse_screen_pos = Vec2::new(18.0, 18.0);
        input.prev_mouse_world_pos = Vec2::new(18.0, 18.0);
        input.left_press_screen_pos = Vec2::new(5.0, 5.0);
        input.left_press_world_pos = Vec2::new(5.0, 5.0);

        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        left.held_duration = 0.1;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert!(!state.is_dragging);
        assert_eq!(state.drag_delta_screen, Vec2::ZERO);
    }

    #[test]
    fn test_rect_scroll_when_hovered() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        input.scroll_delta = 2.5;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert_eq!(state.scroll_delta, 2.5);
    }

    #[test]
    fn test_rect_scroll_zero_when_not_hovered() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        input.scroll_delta = 2.5;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert_eq!(state.scroll_delta, 0.0);
    }

    #[test]
    fn test_rect_screen_uses_screen_coords() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.mouse_world_pos = Vec2::new(100.0, 100.0);
        input.prev_mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.prev_mouse_world_pos = Vec2::new(100.0, 100.0);

        let rect = make_rect();
        let screen_state = input.test_rect_screen(&rect);
        let world_state = input.test_rect_world(&rect);
        assert!(screen_state.hovered);
        assert!(!world_state.hovered);
    }

    #[test]
    fn test_rect_drag_origin() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(20.0, 20.0);
        input.mouse_world_pos = Vec2::new(20.0, 20.0);
        input.prev_mouse_screen_pos = Vec2::new(18.0, 18.0);
        input.prev_mouse_world_pos = Vec2::new(18.0, 18.0);
        input.left_press_screen_pos = Vec2::new(12.0, 14.0);
        input.left_press_world_pos = Vec2::new(12.0, 14.0);

        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        left.held_duration = 0.1;

        let rect = make_rect();
        let state = input.test_rect_world(&rect);
        assert_eq!(state.drag_origin_screen, Vec2::new(12.0, 14.0));
        assert_eq!(state.drag_origin_world, Vec2::new(12.0, 14.0));
    }
}

pub(crate) fn translate_key(key: winit::keyboard::PhysicalKey) -> KeyCode {
    use winit::keyboard::{KeyCode as WK, PhysicalKey};
    match key {
        PhysicalKey::Code(code) => match code {
            WK::KeyA => KeyCode::A,
            WK::KeyB => KeyCode::B,
            WK::KeyC => KeyCode::C,
            WK::KeyD => KeyCode::D,
            WK::KeyE => KeyCode::E,
            WK::KeyF => KeyCode::F,
            WK::KeyG => KeyCode::G,
            WK::KeyH => KeyCode::H,
            WK::KeyI => KeyCode::I,
            WK::KeyJ => KeyCode::J,
            WK::KeyK => KeyCode::K,
            WK::KeyL => KeyCode::L,
            WK::KeyM => KeyCode::M,
            WK::KeyN => KeyCode::N,
            WK::KeyO => KeyCode::O,
            WK::KeyP => KeyCode::P,
            WK::KeyQ => KeyCode::Q,
            WK::KeyR => KeyCode::R,
            WK::KeyS => KeyCode::S,
            WK::KeyT => KeyCode::T,
            WK::KeyU => KeyCode::U,
            WK::KeyV => KeyCode::V,
            WK::KeyW => KeyCode::W,
            WK::KeyX => KeyCode::X,
            WK::KeyY => KeyCode::Y,
            WK::KeyZ => KeyCode::Z,
            WK::Digit0 => KeyCode::Key0,
            WK::Digit1 => KeyCode::Key1,
            WK::Digit2 => KeyCode::Key2,
            WK::Digit3 => KeyCode::Key3,
            WK::Digit4 => KeyCode::Key4,
            WK::Digit5 => KeyCode::Key5,
            WK::Digit6 => KeyCode::Key6,
            WK::Digit7 => KeyCode::Key7,
            WK::Digit8 => KeyCode::Key8,
            WK::Digit9 => KeyCode::Key9,
            WK::Space => KeyCode::Space,
            WK::Enter => KeyCode::Enter,
            WK::Escape => KeyCode::Escape,
            WK::Backspace => KeyCode::Backspace,
            WK::Tab => KeyCode::Tab,
            WK::ArrowLeft => KeyCode::Left,
            WK::ArrowRight => KeyCode::Right,
            WK::ArrowUp => KeyCode::Up,
            WK::ArrowDown => KeyCode::Down,
            WK::ShiftLeft => KeyCode::LShift,
            WK::ShiftRight => KeyCode::RShift,
            WK::ControlLeft => KeyCode::LCtrl,
            WK::ControlRight => KeyCode::RCtrl,
            WK::AltLeft => KeyCode::LAlt,
            WK::AltRight => KeyCode::RAlt,
            WK::F1 => KeyCode::F1,
            WK::F2 => KeyCode::F2,
            WK::F3 => KeyCode::F3,
            WK::F4 => KeyCode::F4,
            WK::F5 => KeyCode::F5,
            WK::F6 => KeyCode::F6,
            WK::F7 => KeyCode::F7,
            WK::F8 => KeyCode::F8,
            WK::F9 => KeyCode::F9,
            WK::F10 => KeyCode::F10,
            WK::F11 => KeyCode::F11,
            WK::F12 => KeyCode::F12,
            _ => KeyCode::Unknown,
        },
        _ => KeyCode::Unknown,
    }
}
