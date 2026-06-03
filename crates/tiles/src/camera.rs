use glam::{Mat4, Vec2};

pub struct Camera {
    pub position: Vec2,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            viewport_width,
            viewport_height,
        }
    }

    pub fn projection(&self, window_width: f32, window_height: f32) -> (Mat4, Vec2, Vec2) {
        let scale = self.fit_scale(window_width, window_height);
        let viewport_screen_w = self.viewport_width * scale;
        let viewport_screen_h = self.viewport_height * scale;
        let offset_x = (window_width - viewport_screen_w) / 2.0;
        let offset_y = (window_height - viewport_screen_h) / 2.0;

        let half_w = self.viewport_width / 2.0;
        let half_h = self.viewport_height / 2.0;

        let left = self.position.x - half_w;
        let right = self.position.x + half_w;
        let bottom = self.position.y - half_h;
        let top = self.position.y + half_h;

        let proj = Mat4::orthographic_rh(left, right, bottom, top, -1000.0, 1000.0);
        let viewport_offset = Vec2::new(offset_x, offset_y);
        let viewport_size = Vec2::new(viewport_screen_w, viewport_screen_h);

        (proj, viewport_offset, viewport_size)
    }

    pub fn fit_scale(&self, window_width: f32, window_height: f32) -> f32 {
        let scale_x = window_width / self.viewport_width;
        let scale_y = window_height / self.viewport_height;
        scale_x.min(scale_y)
    }

    pub fn screen_to_world(&self, screen_pos: Vec2, window_width: f32, window_height: f32) -> Vec2 {
        let scale = self.fit_scale(window_width, window_height);
        let viewport_screen_w = self.viewport_width * scale;
        let viewport_screen_h = self.viewport_height * scale;
        let offset_x = (window_width - viewport_screen_w) / 2.0;
        let offset_y = (window_height - viewport_screen_h) / 2.0;

        let local_x = (screen_pos.x - offset_x) / scale;
        let local_y = self.viewport_height - (screen_pos.y - offset_y) / scale;

        Vec2::new(
            local_x + self.position.x - self.viewport_width / 2.0,
            local_y + self.position.y - self.viewport_height / 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_scale_width_limited() {
        let cam = Camera::new(100.0, 50.0);
        let scale = cam.fit_scale(200.0, 200.0);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn fit_scale_height_limited() {
        let cam = Camera::new(50.0, 100.0);
        let scale = cam.fit_scale(200.0, 200.0);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn fit_scale_exact_match() {
        let cam = Camera::new(100.0, 100.0);
        let scale = cam.fit_scale(100.0, 100.0);
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn projection_centered_viewport() {
        let cam = Camera::new(100.0, 100.0);
        let (_, offset, size) = cam.projection(200.0, 200.0);
        assert_eq!(offset, Vec2::new(0.0, 0.0));
        assert_eq!(size, Vec2::new(200.0, 200.0));
    }

    #[test]
    fn projection_letterboxed() {
        let cam = Camera::new(100.0, 50.0);
        let (_, offset, size) = cam.projection(200.0, 200.0);
        assert_eq!(size, Vec2::new(200.0, 100.0));
        assert_eq!(offset, Vec2::new(0.0, 50.0));
    }

    #[test]
    fn screen_to_world_center() {
        let cam = Camera::new(100.0, 100.0);
        let world = cam.screen_to_world(Vec2::new(50.0, 50.0), 100.0, 100.0);
        assert!((world.x - 0.0).abs() < 1e-5);
        assert!((world.y - 0.0).abs() < 1e-5);
    }

    #[test]
    fn screen_to_world_top_left() {
        let cam = Camera::new(100.0, 100.0);
        let world = cam.screen_to_world(Vec2::new(0.0, 0.0), 100.0, 100.0);
        assert!((world.x - -50.0).abs() < 1e-5);
        assert!((world.y - 50.0).abs() < 1e-5);
    }

    #[test]
    fn screen_to_world_with_camera_offset() {
        let mut cam = Camera::new(100.0, 100.0);
        cam.position = Vec2::new(10.0, 20.0);
        let world = cam.screen_to_world(Vec2::new(50.0, 50.0), 100.0, 100.0);
        assert!((world.x - 10.0).abs() < 1e-5);
        assert!((world.y - 20.0).abs() < 1e-5);
    }

    #[test]
    fn projection_produces_orthographic_matrix() {
        let cam = Camera::new(100.0, 100.0);
        let (proj, _, _) = cam.projection(100.0, 100.0);
        let center = proj * glam::Vec4::new(0.0, 0.0, 0.0, 1.0);
        assert!((center.x - 0.0).abs() < 1e-5);
        assert!((center.y - 0.0).abs() < 1e-5);
    }
}
