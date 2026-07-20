use bytemuck::{Pod, Zeroable};
use glam::dcamera::lh::view::look_to_mat4;
use glam::{DMat4, DVec3, IVec2, Mat4, Vec2, Vec3, Vec4};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages};
use winit::keyboard::KeyCode;
use crate::buffer::TypedBuffer;
use crate::render_context::RenderContext;

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
    pub  position: DVec3,
    yaw: f64,
    pitch: f64,

    direction: IVec2,

    uniform: CameraUniform,
    buffer: TypedBuffer<CameraUniform>,
    layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Camera {
    pub fn new<V: Into<DVec3>, Y: Into<f64>, P: Into<f64>>(context: &RenderContext, position: V, yaw: Y, pitch: P) -> Self {
        let position = position.into();

        let uniform = CameraUniform {
            position: position.extend(0.0).as_vec4(),
            rotation: Mat4::IDENTITY,
        };
        let buffer = TypedBuffer::new_uniform(context, uniform);
        let layout =
            context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Camera Bind Group Layout"),
                    entries: &[buffer.as_layout_entry(0, ShaderStages::FRAGMENT)],
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
            direction: IVec2::splat(0),
            uniform,
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
            DVec3::new(cos_pitch * cos_yaw, sin_pitch.into(), cos_pitch * sin_yaw).normalize(),
            DVec3::Y,
        )
    }

    pub fn process_key_event(&mut self, key_code: KeyCode, is_pressed: bool) {
        let amount = if is_pressed { 1 } else { 0 };
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.direction.y = amount;
            },
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.direction.y = -amount;
            },
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.direction.x = amount;
            },
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.direction.x = -amount;
            },
            _ => {}
        }
    }

    pub fn update(&mut self, context: &RenderContext, delta_time: f64) {
        const SPEED: f64 = 1.0;

        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = DVec3::new(yaw_sin, 0.0, -yaw_cos).normalize();
        let right = DVec3::new(-yaw_cos, 0.0, -yaw_sin).normalize();
        self.position += forward * self.direction.y as f64 * SPEED * delta_time;
        self.position += right * self.direction.x as f64 * SPEED * delta_time;

        self.uniform = CameraUniform {
            position: self.position.extend(0.0).as_vec4(),
            rotation: Mat4::IDENTITY,
        };
        self.buffer.update(context, self.uniform);
    }

    pub fn get_layout(&self) -> BindGroupLayout {
        self.layout.clone()
    }

    pub fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
