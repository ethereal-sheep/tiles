use glam::{Mat4, Vec2};

use crate::Rect;

pub(crate) struct Camera {
    x: f32,
    y: f32,
    w: u32,
    h: u32,
}

impl Camera {
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: viewport_width,
            h: viewport_height,
        }
    }

    pub fn projection(&self, window_width: u32, window_height: u32) -> (Mat4, Vec2, Vec2) {
        let scale = self.fit_scale(window_width, window_height);
        let viewport_screen_w = self.w as f32 * scale;
        let viewport_screen_h = self.h as f32 * scale;
        let offset_x = (window_width as f32 - viewport_screen_w) / 2.0;
        let offset_y = (window_height as f32 - viewport_screen_h) / 2.0;

        let half_w = self.w as f32 / 2.0;
        let half_h = self.h as f32 / 2.0;

        let left = self.x - half_w;
        let right = self.x + half_w;
        let bottom = self.y - half_h;
        let top = self.y + half_h;

        let proj = Mat4::orthographic_rh(left, right, bottom, top, -1000.0, 1000.0);
        let viewport_offset = Vec2::new(offset_x, offset_y);
        let viewport_size = Vec2::new(viewport_screen_w, viewport_screen_h);

        (proj, viewport_offset, viewport_size)
    }

    fn fit_scale(&self, window_width: u32, window_height: u32) -> f32 {
        let scale_x = window_width as f32 / self.w as f32;
        let scale_y = window_height as f32 / self.h as f32;
        scale_x.min(scale_y)
    }

    // --- Coordinate conversion ---
    pub fn pixel_to_world(
        &self,
        window_width: u32,
        window_height: u32,
        screen_x: f32,
        screen_y: f32,
    ) -> (f32, f32) {
        let scale = self.fit_scale(window_width, window_height);
        let viewport_screen_w = self.w as f32 * scale;
        let viewport_screen_h = self.h as f32 * scale;
        let offset_x = (window_width as f32 - viewport_screen_w) / 2.0;
        let offset_y = (window_height as f32 - viewport_screen_h) / 2.0;

        let local_x = (screen_x - offset_x) / scale;
        let local_y = self.h as f32 - (screen_y - offset_y) / scale;

        (
            local_x + self.x - self.w as f32 / 2.0,
            local_y + self.y - self.h as f32 / 2.0,
        )
    }

    pub fn pixel_to_screen(
        &self,
        window_width: u32,
        window_height: u32,
        pixel_x: f32,
        pixel_y: f32,
    ) -> (f32, f32) {
        let scale = self.fit_scale(window_width, window_height);
        let viewport_screen_w = self.w as f32 * scale;
        let viewport_screen_h = self.h as f32 * scale;
        let offset_x = (window_width as f32 - viewport_screen_w) / 2.0;
        let offset_y = (window_height as f32 - viewport_screen_h) / 2.0;
        let vx = (pixel_x - offset_x) / scale;
        let vy = (pixel_y - offset_y) / scale;
        (vx, vy)
    }

    pub fn world_to_screen(&self, world_x: f32, world_y: f32) -> (f32, f32) {
        let half_w = self.w as f32 / 2.0;
        let half_h = self.h as f32 / 2.0;
        let x = world_x - self.x + half_w;
        let y = half_h - (world_y - self.y);
        (x, y)
    }

    pub fn screen_to_world(&self, screen_x: f32, screen_y: f32) -> (f32, f32) {
        let half_w = self.w as f32 / 2.0;
        let half_h = self.h as f32 / 2.0;
        let x = screen_x + self.x - half_w;
        let y = self.y + half_h - screen_y;
        (x, y)
    }

    pub fn world_space_manifold(&self) -> Rect {
        let half_w = self.w as f32 / 2.0;
        let half_h = self.h as f32 / 2.0;
        Rect::from_top_left(self.x - half_w, self.y - half_h, self.w, self.h)
    }

    pub fn screen_space_manifold(&self) -> Rect {
        Rect::from_top_left(0.0, 0.0, self.w, self.h)
    }

    // --- Camera ---
    pub fn position(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn viewport_width(&self) -> u32 {
        self.w
    }

    pub fn viewport_height(&self) -> u32 {
        self.h
    }

    pub fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.w = width;
        self.h = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_scale_width_limited() {
        let cam = Camera::new(100, 50);
        let scale = cam.fit_scale(200, 200);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn fit_scale_height_limited() {
        let cam = Camera::new(50, 100);
        let scale = cam.fit_scale(200, 200);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn fit_scale_exact_match() {
        let cam = Camera::new(100, 100);
        let scale = cam.fit_scale(100, 100);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn projection_centered_viewport() {
        let cam = Camera::new(100, 100);
        let (_, offset, size) = cam.projection(200, 200);
        assert_eq!(offset, Vec2::new(0.0, 0.0));
        assert_eq!(size, Vec2::new(200.0, 200.0));
    }

    #[test]
    fn projection_letterboxed() {
        let cam = Camera::new(100, 50);
        let (_, offset, size) = cam.projection(200, 200);
        assert_eq!(size, Vec2::new(200.0, 100.0));
        assert_eq!(offset, Vec2::new(0.0, 50.0));
    }

    #[test]
    fn pixel_to_world_center() {
        let cam = Camera::new(100, 100);
        let (world_x, world_y) = cam.pixel_to_world(100, 100, 50.0, 50.0);
        assert!((world_x - 0.0).abs() < 1e-5);
        assert!((world_y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_to_world_top_left() {
        let cam = Camera::new(100, 100);
        let (world_x, world_y) = cam.pixel_to_world(100, 100, 0.0, 0.0);
        assert!((world_x - -50.0).abs() < 1e-5);
        assert!((world_y - 50.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_to_world_with_camera_offset() {
        let mut cam = Camera::new(100, 100);
        cam.set_position(10.0, 20.0);
        let (world_x, world_y) = cam.pixel_to_world(100, 100, 50.0, 50.0);
        assert!((world_x - 10.0).abs() < 1e-5);
        assert!((world_y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn projection_produces_orthographic_matrix() {
        let cam = Camera::new(100, 100);
        let (proj, _, _) = cam.projection(100, 100);
        let center = proj * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((center.x - 0.0).abs() < 1e-5);
        assert!((center.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn world_space_manifold_centered_camera() {
        let cam = Camera::new(100, 80);
        let rect = cam.world_space_manifold();
        assert_eq!(rect.x(), -50.0);
        assert_eq!(rect.y(), -40.0);
        assert_eq!(rect.width(), 100);
        assert_eq!(rect.height(), 80);
    }

    #[test]
    fn world_space_manifold_with_camera_offset() {
        let mut cam = Camera::new(100, 100);
        cam.set_position(20.0, 30.0);
        let rect = cam.world_space_manifold();
        assert_eq!(rect.x(), -30.0);
        assert_eq!(rect.y(), -20.0);
        assert_eq!(rect.width(), 100);
        assert_eq!(rect.height(), 100);
    }

    #[test]
    fn screen_space_manifold_always_at_origin() {
        let cam = Camera::new(200, 150);
        let rect = cam.screen_space_manifold();
        assert_eq!(rect.x(), 0.0);
        assert_eq!(rect.y(), 0.0);
        assert_eq!(rect.width(), 200);
        assert_eq!(rect.height(), 150);
    }

    #[test]
    fn screen_space_manifold_ignores_camera_position() {
        let mut cam = Camera::new(64, 64);
        cam.set_position(100.0, -50.0);
        let rect = cam.screen_space_manifold();
        assert_eq!(rect.x(), 0.0);
        assert_eq!(rect.y(), 0.0);
        assert_eq!(rect.width(), 64);
        assert_eq!(rect.height(), 64);
    }

    #[test]
    fn screen_to_world_center() {
        let cam = Camera::new(100, 100);
        let (x, y) = cam.screen_to_world(50.0, 50.0);
        assert!((x - 0.0).abs() < 1e-5);
        assert!((y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn screen_to_world_top_left() {
        let cam = Camera::new(100, 100);
        let (x, y) = cam.screen_to_world(0.0, 0.0);
        assert!((x - -50.0).abs() < 1e-5);
        assert!((y - 50.0).abs() < 1e-5);
    }

    #[test]
    fn screen_to_world_with_camera_offset() {
        let mut cam = Camera::new(100, 100);
        cam.set_position(10.0, 20.0);
        let (x, y) = cam.screen_to_world(50.0, 50.0);
        assert!((x - 10.0).abs() < 1e-5);
        assert!((y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn screen_to_world_roundtrip() {
        let mut cam = Camera::new(200, 150);
        cam.set_position(15.0, -10.0);
        let (sx, sy) = cam.world_to_screen(30.0, 40.0);
        let (wx, wy) = cam.screen_to_world(sx, sy);
        assert!((wx - 30.0).abs() < 1e-5);
        assert!((wy - 40.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_to_screen_center_scaled_window() {
        let cam = Camera::new(100, 100);
        let (sx, sy) = cam.pixel_to_screen(200, 200, 100.0, 100.0);
        assert!((sx - 50.0).abs() < 1e-5);
        assert!((sy - 50.0).abs() < 1e-5);
    }

    #[test]
    fn pixel_to_screen_letterboxed() {
        let cam = Camera::new(100, 50);
        // window 200x200, viewport fits at scale 2 → 200x100 centered with 50px vertical offset
        let (sx, sy) = cam.pixel_to_screen(200, 200, 100.0, 75.0);
        assert!((sx - 50.0).abs() < 1e-5);
        assert!((sy - 12.5).abs() < 1e-5);
    }

    #[test]
    fn pixel_to_screen_top_left_of_viewport() {
        let cam = Camera::new(100, 50);
        // viewport offset is (0, 50) in a 200x200 window
        let (sx, sy) = cam.pixel_to_screen(200, 200, 0.0, 50.0);
        assert!((sx - 0.0).abs() < 1e-5);
        assert!((sy - 0.0).abs() < 1e-5);
    }
}
