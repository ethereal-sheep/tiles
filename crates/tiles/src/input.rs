use glam::Vec2;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    Space, Enter, Escape, Backspace, Tab,
    Left, Right, Up, Down,
    LShift, RShift, LCtrl, RCtrl, LAlt, RAlt,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
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
    Moved { screen_delta: Vec2, world_delta: Vec2 },
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
    pub scroll_delta: f32,
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
            scroll_delta: 0.0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.scroll_delta = 0.0;
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
