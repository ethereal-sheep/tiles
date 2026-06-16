use glam::Vec2;
use std::collections::HashMap;

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
}

pub(crate) struct InputState {
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

    pub fn begin_state_update(&mut self) {
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
}

pub const HOLD_THRESHOLD_SECS: f32 = 0.4;
pub const DOUBLE_CLICK_THRESHOLD_SECS: f32 = 0.25;

#[derive(Debug, Clone)]
pub(crate) struct ButtonState {
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
        if self.is_down {
            self.held_duration += dt;
        }

        if self.pressed_this_frame {
            self.held_duration = 0.0;
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
        assert_eq!(s.held_duration, 0.0);
    }

    #[test]
    fn input_state_key_queries() {
        let mut input = InputState::new();

        assert!(!input.is_key_down(KeyCode::A));
        assert!(!input.is_key_pressed(KeyCode::A));

        // Simulate pressing A
        let state = input
            .keys_states
            .entry(KeyCode::A)
            .or_insert(ButtonState::new());
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
        input.begin_state_update();
        assert_eq!(input.scroll_delta, 0.0);
    }

    #[test]
    fn input_state_update_propagates_to_button_states() {
        let mut input = InputState::new();

        let state = input
            .keys_states
            .entry(KeyCode::Space)
            .or_insert(ButtonState::new());
        state.pressed_this_frame = true;
        state.is_down = true;

        input.update(0.016, 1.0);

        assert_eq!(input.keys_states[&KeyCode::Space].held_duration, 0.0);
    }

    #[test]
    fn input_state_reset_clears_all_frame_flags() {
        let mut input = InputState::new();

        let state = input
            .keys_states
            .entry(KeyCode::W)
            .or_insert(ButtonState::new());
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
}

#[cfg(feature = "runtime")]
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
