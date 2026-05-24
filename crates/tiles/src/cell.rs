use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct CellInstance {
    pub position: [f32; 3],
    pub _pad0: f32,
    pub color: [f32; 4],
    pub rotation: [f32; 4],
    pub emissive: f32,
    pub _pad1: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightData {
    pub position: [f32; 2],
    pub radius: f32,
    pub intensity: f32,
    pub color: [f32; 3],
    pub _pad: f32,
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

    pub fn color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    pub fn rgba(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
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

    pub fn rgb8(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = [
            srgb_to_linear(r as f32 / 255.0),
            srgb_to_linear(g as f32 / 255.0),
            srgb_to_linear(b as f32 / 255.0),
            1.0,
        ];
        self
    }

    pub fn rgba8(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color = [
            srgb_to_linear(r as f32 / 255.0),
            srgb_to_linear(g as f32 / 255.0),
            srgb_to_linear(b as f32 / 255.0),
            a as f32 / 255.0,
        ];
        self
    }

    pub fn hex(mut self, rgb: u32) -> Self {
        let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
        let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
        let b = (rgb & 0xFF) as f32 / 255.0;
        self.color = [srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), 1.0];
        self
    }

    pub(crate) fn to_instance(&self) -> CellInstance {
        let q = self.quat;
        CellInstance {
            position: self.position.to_array(),
            _pad0: 0.0,
            color: self.color,
            rotation: [q.x, q.y, q.z, q.w],
            emissive: if self.light_radius >= 0.0 { 1.0 } else { 0.0 },
            _pad1: [0.0; 3],
        }
    }

    pub(crate) fn to_light_data(&self) -> LightData {
        LightData {
            position: [self.position.x, self.position.y],
            radius: self.light_radius.max(0.0),
            intensity: self.intensity,
            color: [self.color[0], self.color[1], self.color[2]],
            _pad: 0.0,
        }
    }

    pub(crate) fn is_opaque(&self) -> bool {
        self.color[3] >= 1.0
    }
}

fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}
