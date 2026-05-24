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
