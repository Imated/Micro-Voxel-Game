use std::collections::HashSet;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use winit::keyboard::KeyCode;

use crate::world::ChunkPos;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
pub struct TransformUniform {
    pub position: Vec4,
    pub rotation: Mat4,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
}

impl Transform {
    #[must_use]
    pub fn rotation_matrix(&self) -> Mat4 {
        Mat4::from_rotation_y(self.rotation.y) * Mat4::from_rotation_x(self.rotation.x)
    }

    #[must_use]
    pub fn transformation_uniform(&self) -> TransformUniform {
        TransformUniform {
            position: self.position.extend(0.0),
            rotation: self.rotation_matrix(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputHandler {
    pub pressed_keys: HashSet<KeyCode>,
    pub mouse_delta: Vec2,
}

impl InputHandler {
    #[must_use]
    pub fn is_key_down(&self, key_code: KeyCode) -> bool {
        self.pressed_keys.contains(&key_code)
    }
}

pub struct ChunkLoader {
    pub radius: u16,
    pub previous_chunk: ChunkPos,
}
