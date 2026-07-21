use std::time::Duration;
use crate::buffer::TypedBuffer;
use crate::render_context::RenderContext;
use bytemuck::{Pod, Zeroable};
use glam::dcamera::lh::view::look_to_mat4;
use glam::{DMat4, DVec2, DVec3, IVec2, Mat4, Vec2, Vec3, Vec4};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};
use winit::keyboard::KeyCode;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
pub struct CameraUniform {
    pub position: Vec4,
    pub rotation: Mat4,
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    yaw: f32,
    pitch: f32,

    move_forward: bool,
    move_back: bool,
    move_left: bool,
    move_right: bool,

    buffer: TypedBuffer<CameraUniform>,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Camera {
    pub fn new<V: Into<Vec3>, Y: Into<f32>, P: Into<f32>>(
        context: &RenderContext,
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        let position = position.into();

        let uniform = CameraUniform {
            position: position.extend(0.0),
            rotation: Mat4::IDENTITY,
        };
        let buffer = TypedBuffer::new_uniform(context, uniform);
        let layout = context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[buffer.as_layout_entry(0, ShaderStages::COMPUTE)],
            });
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &layout,
            entries: &[buffer.as_bind_group_entry(0)],
        });

        Self {
            position,
            yaw: yaw.into(),
            pitch: pitch.into(),
            move_forward: false,
            move_back: false,
            move_left: false,
            move_right: false,
            buffer,
            layout,
            bind_group,
        }
    }

    pub fn calc_matrix(&self) -> DMat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        look_to_mat4(
            self.position.into(),
            DVec3::new((cos_pitch * cos_yaw) as f64, sin_pitch.into(), (cos_pitch * sin_yaw) as f64).normalize(),
            DVec3::Y,
        )
    }

    pub fn process_key_event(&mut self, key_code: KeyCode, is_pressed: bool) {
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.move_forward = is_pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.move_back = is_pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.move_right = is_pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.move_left = is_pressed,
            _ => {}
        }
    }

    pub fn update(&mut self, context: &RenderContext, delta_time: Duration) {
        const SPEED: f32 = 1.0;

        let direction = Vec2::new(
            (self.move_right as i8 - self.move_left as i8) as f32,
            (self.move_forward as i8 - self.move_back as i8) as f32,
        )
        .normalize_or_zero();

        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = Vec3::new(yaw_sin, 0.0, yaw_cos).normalize();
        let right = Vec3::new(-yaw_cos, 0.0, -yaw_sin).normalize();
        self.position += forward * direction.y * SPEED * delta_time.as_secs_f32();
        self.position += right * direction.x * SPEED * delta_time.as_secs_f32();

        self.buffer.update(
            context,
            CameraUniform {
                position: self.position.extend(0.0),
                rotation: Mat4::IDENTITY,
            },
        );
    }

    pub fn get_layout(&self) -> BindGroupLayout {
        self.layout.clone()
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
