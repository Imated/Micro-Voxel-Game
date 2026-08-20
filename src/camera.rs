use crate::buffer::TypedBuffer;
use crate::render_context::RenderContext;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
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
    move_up: bool,
    move_down: bool,
    mouse_delta: Vec2,

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
            move_up: false,
            move_down: false,
            mouse_delta: Vec2::default(),
            buffer,
            layout,
            bind_group,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::from_rotation_y(self.yaw) * Mat4::from_rotation_x(self.pitch)
    }

    pub fn process_key_event(&mut self, key_code: KeyCode, is_pressed: bool) {
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.move_forward = is_pressed,
            KeyCode::KeyS | KeyCode::ArrowDown => self.move_back = is_pressed,
            KeyCode::KeyD | KeyCode::ArrowRight => self.move_right = is_pressed,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.move_left = is_pressed,
            KeyCode::Space => self.move_up = is_pressed,
            KeyCode::ShiftLeft => self.move_down = is_pressed,
            _ => {}
        }
    }

    pub fn process_mouse_movement(&mut self, delta: Vec2) {
        self.mouse_delta += delta;
    }

    pub fn update(&mut self, context: &RenderContext, delta_time: Duration) {
        const SPEED: f32 = 100.0;
        const SENSITIVITY: f32 = 0.001;

        let direction = Vec2::new(
            (self.move_right as i8 - self.move_left as i8) as f32,
            (self.move_forward as i8 - self.move_back as i8) as f32,
        )
        .normalize_or_zero();

        let rotation = self.calc_matrix();
        let forward = -rotation.z_axis.truncate().with_y(0.0).normalize_or_zero();
        let right = rotation.x_axis.truncate().with_y(0.0).normalize_or_zero();
        self.position += forward * direction.y * SPEED * delta_time.as_secs_f32();
        self.position += right * direction.x * SPEED * delta_time.as_secs_f32();

        self.position.y +=
            (self.move_up as i8 - self.move_down as i8) as f32 * SPEED * delta_time.as_secs_f32();

        self.yaw -= self.mouse_delta.x * SENSITIVITY;
        self.pitch -= self.mouse_delta.y * SENSITIVITY;

        self.mouse_delta = Vec2::ZERO;

        // -/+ 0.001 so when u look all the way down or all the way up it doesn't invert forward vector
        self.pitch = self.pitch.clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);

        self.buffer.update(
            context,
            CameraUniform {
                position: self.position.extend(0.0),
                rotation,
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
