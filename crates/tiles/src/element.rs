use glam::Vec2;

use crate::drawable::Drawable;
use crate::input::{InputState, MouseButton, HOLD_THRESHOLD_SECS};
use crate::runner::State;
use crate::shape::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    Default,
    Hovered,
    Pressed,
    Captured,
}

#[derive(Debug, Clone, Copy)]
pub struct DragInfo {
    pub delta_screen: Vec2,
    pub delta_world: Vec2,
    pub total_delta_screen: Vec2,
    pub total_delta_world: Vec2,
    pub origin_screen: Vec2,
    pub origin_world: Vec2,
    pub elapsed: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HitState {
    is_hovered: bool,
    has_entered: bool,
    has_left: bool,
    is_down: bool,
    is_drag_end: bool,
    is_pressed: bool,
    is_released: bool,
    is_clicked: bool,
    is_double_clicked: bool,
    held_duration: f32,
    is_released_after_hold: bool,
    is_right_clicked: bool,
    is_middle_clicked: bool,
    drag_delta_screen: Vec2,
    drag_delta_world: Vec2,
    drag_total_delta_screen: Vec2,
    drag_total_delta_world: Vec2,
    drag_origin_screen: Vec2,
    drag_origin_world: Vec2,
    scroll_delta: f32,
}

impl HitState {
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    pub fn has_entered(&self) -> bool {
        self.has_entered
    }

    pub fn has_left(&self) -> bool {
        self.has_left
    }

    pub fn is_down(&self) -> bool {
        self.is_down
    }

    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    pub fn is_released(&self) -> bool {
        self.is_released
    }

    pub fn is_clicked(&self) -> bool {
        self.is_clicked
    }

    pub fn is_double_clicked(&self) -> bool {
        self.is_double_clicked
    }

    pub fn is_held(&self) -> Option<f32> {
        if self.is_down && self.held_duration >= HOLD_THRESHOLD_SECS {
            Some(self.held_duration)
        } else {
            None
        }
    }

    pub fn is_released_after_hold(&self) -> bool {
        self.is_released_after_hold
    }

    pub fn is_dragging(&self) -> Option<DragInfo> {
        self.is_down.then(|| DragInfo {
            delta_screen: self.drag_delta_screen,
            delta_world: self.drag_delta_world,
            total_delta_screen: self.drag_total_delta_screen,
            total_delta_world: self.drag_total_delta_world,
            origin_screen: self.drag_origin_screen,
            origin_world: self.drag_origin_world,
            elapsed: self.held_duration,
        })
    }

    pub fn is_drag_end(&self) -> Option<DragInfo> {
        self.is_drag_end.then(|| DragInfo {
            delta_screen: self.drag_delta_screen,
            delta_world: self.drag_delta_world,
            total_delta_screen: self.drag_total_delta_screen,
            total_delta_world: self.drag_total_delta_world,
            origin_screen: self.drag_origin_screen,
            origin_world: self.drag_origin_world,
            elapsed: self.held_duration,
        })
    }

    pub fn is_right_clicked(&self) -> bool {
        self.is_right_clicked
    }

    pub fn is_middle_clicked(&self) -> bool {
        self.is_middle_clicked
    }

    pub fn is_scrolling(&self) -> Option<f32> {
        if self.scroll_delta != 0.0 {
            Some(self.scroll_delta)
        } else {
            None
        }
    }
}

pub(crate) fn test_shape(input: &InputState, shape: &impl Shape, is_screen: bool) -> HitState {
    let mouse_pos = if is_screen {
        input.mouse_screen_pos
    } else {
        input.mouse_world_pos
    };

    let prev_mouse_pos = if is_screen {
        input.prev_mouse_screen_pos
    } else {
        input.prev_mouse_world_pos
    };

    let press_pos = if is_screen {
        input.left_press_screen_pos
    } else {
        input.left_press_world_pos
    };

    let is_hovered = shape.contains_point(mouse_pos.x, mouse_pos.y);
    let was_hovered = shape.contains_point(prev_mouse_pos.x, prev_mouse_pos.y);
    let has_entered = is_hovered && !was_hovered;
    let has_left = !is_hovered && was_hovered;

    let left = input.mouse_buttons_states.get(&MouseButton::Left);
    let left_down = left.is_some_and(|s| s.is_down());
    let left_pressed_this_frame = left.is_some_and(|s| s.pressed_this_frame);
    let left_released_this_frame = left.is_some_and(|s| s.released_this_frame);
    let left_held_duration = left.map_or(0.0, |s| s.held_duration);

    let press_was_inside = shape.contains_point(press_pos.x, press_pos.y);

    let is_pressed = is_hovered && left_pressed_this_frame;
    let is_released = is_hovered && left_released_this_frame;
    let is_clicked = is_released && is_hovered && left_held_duration < HOLD_THRESHOLD_SECS;
    let is_double_clicked = is_clicked && left.is_some_and(|s| s.press_count >= 2);

    let is_down = left_down && press_was_inside;
    let is_released_after_hold = is_released && left_held_duration >= HOLD_THRESHOLD_SECS;

    let right = input.mouse_buttons_states.get(&MouseButton::Right);
    let is_right_clicked = is_hovered
        && right.is_some_and(|s| s.released_this_frame && s.held_duration < HOLD_THRESHOLD_SECS);

    let middle = input.mouse_buttons_states.get(&MouseButton::Middle);
    let is_middle_clicked = is_hovered
        && middle.is_some_and(|s| s.released_this_frame && s.held_duration < HOLD_THRESHOLD_SECS);

    let drag_delta_screen = input.mouse_screen_pos - input.prev_mouse_screen_pos;
    let drag_delta_world = input.mouse_world_pos - input.prev_mouse_world_pos;
    let drag_origin_screen = input.left_press_screen_pos;
    let drag_origin_world = input.left_press_world_pos;
    let drag_total_delta_screen = input.mouse_screen_pos - input.left_press_screen_pos;
    let drag_total_delta_world = input.mouse_world_pos - input.left_press_world_pos;
    let is_drag_end = left_released_this_frame;

    let scroll_delta = if is_hovered { input.scroll_delta } else { 0.0 };

    HitState {
        is_hovered,
        has_entered,
        has_left,
        is_down,
        is_drag_end,
        is_pressed,
        is_released,
        is_clicked,
        is_double_clicked,
        is_released_after_hold,
        held_duration: left_held_duration,
        is_right_clicked,
        is_middle_clicked,
        drag_delta_screen,
        drag_delta_world,
        drag_total_delta_screen,
        drag_total_delta_world,
        drag_origin_screen,
        drag_origin_world,
        scroll_delta,
    }
}

fn element_state_from(hit: &HitState) -> ElementState {
    if hit.is_down && hit.is_hovered {
        ElementState::Pressed
    } else if hit.is_down {
        ElementState::Captured
    } else if hit.is_hovered {
        ElementState::Hovered
    } else {
        ElementState::Default
    }
}

pub trait Element {
    fn shape(&self) -> impl Shape;
    fn draw(&self, state: ElementState) -> impl Drawable;

    fn handle_screen(&self, state: &mut State) -> HitState {
        let shape = self.shape();
        let hit = state.test_shape_screen(&shape);
        let elem_state = element_state_from(&hit);
        state.draw_screen_overlay(self.draw(elem_state));
        hit
    }

    fn handle_world(&self, state: &mut State) -> HitState {
        let shape = self.shape();
        let hit = state.test_shape_world(&shape);
        let elem_state = element_state_from(&hit);
        state.draw_world_overlay(self.draw(elem_state));
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ButtonState, InputState, MouseButton};
    use crate::rect::Rect;
    use crate::shape::Shape;

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

    fn test_world(input: &InputState, shape: &impl Shape) -> HitState {
        test_shape(input, shape, false)
    }

    fn test_screen(input: &InputState, shape: &impl Shape) -> HitState {
        test_shape(input, shape, true)
    }

    #[test]
    fn hovered_when_inside() {
        let input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(hit.is_hovered());
        assert!(!hit.has_entered());
        assert!(!hit.has_left());
    }

    #[test]
    fn not_hovered_when_outside() {
        let input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(!hit.is_hovered());
    }

    #[test]
    fn just_entered() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        input.prev_mouse_screen_pos = Vec2::new(5.0, 5.0);
        input.prev_mouse_world_pos = Vec2::new(5.0, 5.0);
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(hit.is_hovered());
        assert!(hit.has_entered());
        assert!(!hit.has_left());
    }

    #[test]
    fn just_left() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        input.prev_mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.prev_mouse_world_pos = Vec2::new(15.0, 15.0);
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(!hit.is_hovered());
        assert!(!hit.has_entered());
        assert!(hit.has_left());
    }

    #[test]
    fn pressed_inside() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_pressed());
        assert!(!hit.is_released());
        assert!(!hit.is_clicked());
    }

    #[test]
    fn pressed_outside_not_registered() {
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
        let hit = test_world(&input, &rect);
        assert!(!hit.is_pressed());
    }

    #[test]
    fn click_requires_press_inside() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_released());
        assert!(hit.is_clicked());
        assert!(!hit.is_double_clicked());
    }

    #[test]
    fn click_rejected_when_press_outside() {
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
        let hit = test_world(&input, &rect);
        assert!(!hit.is_released());
        assert!(!hit.is_clicked());
    }

    #[test]
    fn double_click() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_clicked());
        assert!(hit.is_double_clicked());
    }

    #[test]
    fn hold_past_threshold() {
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
        let hit = test_world(&input, &rect);
        assert_eq!(hit.is_held(), Some(0.5));
    }

    #[test]
    fn released_after_hold() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_released_after_hold());
        assert!(!hit.is_clicked());
    }

    #[test]
    fn right_click() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let right = input
            .mouse_buttons_states
            .entry(MouseButton::Right)
            .or_insert(ButtonState::new());
        right.released_this_frame = true;
        right.is_down = false;
        right.held_duration = 0.05;

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(hit.is_right_clicked());
        assert!(!hit.is_middle_clicked());
    }

    #[test]
    fn middle_click() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let middle = input
            .mouse_buttons_states
            .entry(MouseButton::Middle)
            .or_insert(ButtonState::new());
        middle.released_this_frame = true;
        middle.is_down = false;
        middle.held_duration = 0.05;

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(!hit.is_right_clicked());
        assert!(hit.is_middle_clicked());
    }

    #[test]
    fn drag_active() {
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
        let hit = test_world(&input, &rect);
        let drag = hit.is_dragging().unwrap();
        assert_eq!(drag.delta_screen, Vec2::new(2.0, 2.0));
        assert_eq!(drag.delta_world, Vec2::new(2.0, 2.0));
    }

    #[test]
    fn drag_not_active_below_threshold() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_dragging().is_none());
    }

    #[test]
    fn drag_not_active_when_press_outside() {
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
        let hit = test_world(&input, &rect);
        assert!(hit.is_dragging().is_none());
    }

    #[test]
    fn scroll_when_hovered() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        input.scroll_delta = 2.5;

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert_eq!(hit.is_scrolling(), Some(2.5));
    }

    #[test]
    fn scroll_none_when_not_hovered() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        input.scroll_delta = 2.5;

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(hit.is_scrolling().is_none());
    }

    #[test]
    fn screen_uses_screen_coords() {
        let mut input = InputState::new();
        input.mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.mouse_world_pos = Vec2::new(100.0, 100.0);
        input.prev_mouse_screen_pos = Vec2::new(15.0, 15.0);
        input.prev_mouse_world_pos = Vec2::new(100.0, 100.0);

        let rect = make_rect();
        let screen_hit = test_screen(&input, &rect);
        let world_hit = test_world(&input, &rect);
        assert!(screen_hit.is_hovered());
        assert!(!world_hit.is_hovered());
    }

    #[test]
    fn drag_origin() {
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
        let hit = test_world(&input, &rect);
        let drag = hit.is_dragging().unwrap();
        assert_eq!(drag.origin_screen, Vec2::new(12.0, 14.0));
        assert_eq!(drag.origin_world, Vec2::new(12.0, 14.0));
    }

    #[test]
    fn element_state_default_when_idle() {
        let input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert_eq!(element_state_from(&hit), ElementState::Default);
    }

    #[test]
    fn element_state_hovered() {
        let input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert_eq!(element_state_from(&hit), ElementState::Hovered);
    }

    #[test]
    fn element_state_pressed() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert_eq!(element_state_from(&hit), ElementState::Pressed);
    }

    #[test]
    fn element_state_captured() {
        let mut input = input_with_mouse_at(Vec2::new(5.0, 5.0), Vec2::new(5.0, 5.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert_eq!(element_state_from(&hit), ElementState::Captured);
    }

    #[test]
    fn is_down_true_when_press_inside() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(15.0, 15.0);
        input.left_press_world_pos = Vec2::new(15.0, 15.0);

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(hit.is_down());
    }

    #[test]
    fn is_down_false_when_press_outside() {
        let mut input = input_with_mouse_at(Vec2::new(15.0, 15.0), Vec2::new(15.0, 15.0));
        let left = input
            .mouse_buttons_states
            .entry(MouseButton::Left)
            .or_insert(ButtonState::new());
        left.is_down = true;
        input.left_press_screen_pos = Vec2::new(5.0, 5.0);
        input.left_press_world_pos = Vec2::new(5.0, 5.0);

        let rect = make_rect();
        let hit = test_world(&input, &rect);
        assert!(!hit.is_down());
    }
}
