use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};

use crate::color::Color;
use crate::rect::Rect;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CellInstance {
    pub position: [f32; 3],
    pub emissive: f32,
    pub color: [f32; 4],
    pub rotation: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightData {
    pub position: [f32; 2],
    pub radius: f32,
    pub intensity: f32,
    pub color: [f32; 4],
}

#[derive(Clone, Copy)]
pub enum Rotation {
    Z(f32),
    FlipX(f32),
    FlipY(f32),
    DiagonalTL(f32),
    DiagonalTR(f32),
}

impl Rotation {
    pub(crate) fn to_quat(self) -> Quat {
        match self {
            // Z: 0→1 = 0→90°
            Rotation::Z(t) => {
                let angle = t * std::f32::consts::FRAC_PI_2;
                Quat::from_rotation_z(angle)
            }
            // FlipX: 0→1 = 0→180° (rotation around X axis)
            Rotation::FlipX(t) => {
                let angle = t * std::f32::consts::PI;
                Quat::from_rotation_x(angle)
            }
            // FlipY: 0→1 = 0→180° (rotation around Y axis)
            Rotation::FlipY(t) => {
                let angle = t * std::f32::consts::PI;
                Quat::from_rotation_y(angle)
            }
            // DiagonalTL: rotation around top-left to bottom-right axis (1, -1, 0) normalized
            Rotation::DiagonalTL(t) => {
                let angle = t * std::f32::consts::PI;
                let axis = Vec3::new(1.0, -1.0, 0.0).normalize();
                Quat::from_axis_angle(axis, angle)
            }
            // DiagonalTR: rotation around top-right to bottom-left axis (-1, -1, 0) normalized
            Rotation::DiagonalTR(t) => {
                let angle = t * std::f32::consts::PI;
                let axis = Vec3::new(-1.0, -1.0, 0.0).normalize();
                Quat::from_axis_angle(axis, angle)
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub position: Vec3,
    pub color: [f32; 4],
    pub(crate) quat: Quat,
    pub(crate) light_radius: f32,
    pub(crate) intensity: f32,
}

impl Cell {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec3::new(x, y, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
            quat: Quat::IDENTITY,
            light_radius: -1.0,
            intensity: 1.0,
        }
    }

    pub fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Vec3::new(x, y, z),
            color: [1.0, 1.0, 1.0, 1.0],
            quat: Quat::IDENTITY,
            light_radius: -1.0,
            intensity: 1.0,
        }
    }

    pub fn at(pos: glam::Vec2) -> Self {
        Self {
            position: Vec3::new(pos.x, pos.y, 0.0),
            color: [1.0, 1.0, 1.0, 1.0],
            quat: Quat::IDENTITY,
            light_radius: -1.0,
            intensity: 1.0,
        }
    }

    pub fn at_3d(pos: Vec3) -> Self {
        Self {
            position: pos,
            color: [1.0, 1.0, 1.0, 1.0],
            quat: Quat::IDENTITY,
            light_radius: -1.0,
            intensity: 1.0,
        }
    }

    pub fn rotation(mut self, rotation: Rotation) -> Self {
        self.quat = rotation.to_quat();
        self
    }

    pub fn color(mut self, c: Color) -> Self {
        self.color = c.to_array();
        self
    }

    pub fn light(mut self, radius: f32) -> Self {
        self.light_radius = radius;
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    pub fn emissive(self) -> Self {
        self.light(0.0)
    }

    pub(crate) fn to_instance(&self) -> CellInstance {
        let q = self.quat;
        CellInstance {
            position: self.position.to_array(),
            emissive: if self.light_radius >= 0.0 { 1.0 } else { 0.0 },
            color: self.color,
            rotation: [q.x, q.y, q.z, q.w],
        }
    }

    pub(crate) fn to_screen_instance(&self) -> CellInstance {
        let q = self.quat;
        CellInstance {
            position: self.position.to_array(),
            emissive: 1.0,
            color: self.color,
            rotation: [q.x, q.y, q.z, q.w],
        }
    }

    pub(crate) fn to_light_data(&self) -> LightData {
        LightData {
            position: [self.position.x, self.position.y],
            radius: self.light_radius.max(0.0),
            intensity: self.intensity,
            color: self.color,
        }
    }

    pub(crate) fn to_rect(&self) -> Rect {
        Rect::from_top_left(self.position.x, self.position.y, 1, 1)
    }

    pub(crate) fn is_opaque(&self) -> bool {
        self.color[3] >= 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cell_defaults() {
        let c = Cell::new(3.0, 5.0);
        assert_eq!(c.position, Vec3::new(3.0, 5.0, 0.0));
        assert_eq!(c.color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(c.light_radius, -1.0);
        assert_eq!(c.intensity, 1.0);
    }

    #[test]
    fn new_3d_preserves_z() {
        let c = Cell::new_3d(1.0, 2.0, -5.0);
        assert_eq!(c.position.z, -5.0);
    }

    #[test]
    fn color_sets_color() {
        let c = Cell::new(0.0, 0.0).color(Color::linear(0.2, 0.4, 0.6, 0.8));
        assert_eq!(c.color, [0.2, 0.4, 0.6, 0.8]);
    }

    #[test]
    fn is_opaque_check() {
        assert!(Cell::new(0.0, 0.0).is_opaque());
        assert!(!Cell::new(0.0, 0.0)
            .color(Color::linear(1.0, 1.0, 1.0, 0.5))
            .is_opaque());
    }

    #[test]
    fn emissive_sets_light_radius_zero() {
        let c = Cell::new(0.0, 0.0).emissive();
        assert_eq!(c.light_radius, 0.0);
    }

    #[test]
    fn light_sets_radius() {
        let c = Cell::new(0.0, 0.0).light(5.0);
        assert_eq!(c.light_radius, 5.0);
    }

    #[test]
    fn to_instance_non_emissive() {
        let c = Cell::new(1.0, 2.0);
        let i = c.to_instance();
        assert_eq!(i.position, [1.0, 2.0, 0.0]);
        assert_eq!(i.emissive, 0.0);
    }

    #[test]
    fn to_instance_emissive() {
        let c = Cell::new(0.0, 0.0).emissive();
        let i = c.to_instance();
        assert_eq!(i.emissive, 1.0);
    }

    #[test]
    fn to_screen_instance_always_emissive() {
        let c = Cell::new(5.0, 10.0);
        let i = c.to_screen_instance();
        assert_eq!(i.emissive, 1.0);
        assert_eq!(i.position, [5.0, 10.0, 0.0]);
    }

    #[test]
    fn to_light_data_clamps_radius() {
        let c = Cell::new(0.0, 0.0).light(3.0).intensity(2.0);
        let ld = c.to_light_data();
        assert_eq!(ld.radius, 3.0);
        assert_eq!(ld.intensity, 2.0);

        let c2 = Cell::new(0.0, 0.0);
        let ld2 = c2.to_light_data();
        assert_eq!(ld2.radius, 0.0);
    }

    #[test]
    fn hex_color() {
        let c = Cell::new(0.0, 0.0).color(Color::hex(0xFF0000));
        assert!((c.color[0] - 1.0).abs() < 1e-4);
        assert!(c.color[1] < 0.01);
        assert!(c.color[2] < 0.01);
    }
}
