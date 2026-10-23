use crate::ecs::components::{InputHandler, TransformUniform};
use crate::render_context::RenderContext;
use crate::{buffer::TypedBuffer, ecs::components::Transform};
use glam::Vec2;
use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct Camera {
    pub mouse_delta: Vec2,

    pub buffer: TypedBuffer<TransformUniform>,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Camera {
    #[must_use]
    pub fn new(context: &RenderContext) -> Self {
        let buffer = TypedBuffer::new_uniform(context.clone(), TransformUniform::default());
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
            mouse_delta: Vec2::default(),
            buffer,
            layout,
            bind_group,
        }
    }

    #[must_use]
    pub fn get_layout(&self) -> BindGroupLayout {
        self.layout.clone()
    }

    #[must_use]
    pub const fn get_bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

pub fn player_camera_binding(entity_world: &hecs::World) -> Option<BindGroup> {
    if let Some(camera) = entity_world.query::<&Camera>().iter().next() {
        return Some(camera.get_bind_group().clone()); // there shgould only be one
    }
    None
}

pub fn update_player(entity_world: &mut hecs::World, delta_time: Duration, speed: f32) {
    const SENSITIVITY: f32 = 0.001;
    for (transform, camera, input) in
        entity_world.query_mut::<(&mut Transform, &mut Camera, &mut InputHandler)>()
    {
        let direction = Vec2::new(
            f32::from(
                i8::from(
                    input.is_key_down(KeyCode::KeyD) || input.is_key_down(KeyCode::ArrowRight),
                ) - i8::from(
                    input.is_key_down(KeyCode::KeyA) || input.is_key_down(KeyCode::ArrowLeft),
                ),
            ),
            f32::from(
                i8::from(input.is_key_down(KeyCode::KeyW) || input.is_key_down(KeyCode::ArrowUp))
                    - i8::from(
                        input.is_key_down(KeyCode::KeyS) || input.is_key_down(KeyCode::ArrowDown),
                    ),
            ),
        )
        .normalize_or_zero();

        let rotation = transform.rotation_matrix();
        let forward = -rotation.z_axis.truncate().with_y(0.0).normalize_or_zero();
        let right = rotation.x_axis.truncate().with_y(0.0).normalize_or_zero();
        transform.position += forward * direction.y * speed * delta_time.as_secs_f32();
        transform.position += right * direction.x * speed * delta_time.as_secs_f32();

        transform.position.y = (f32::from(
            i8::from(input.is_key_down(KeyCode::Space))
                - i8::from(input.is_key_down(KeyCode::ShiftLeft)),
        ) * speed)
            .mul_add(delta_time.as_secs_f32(), transform.position.y);

        transform.rotation.y = input
            .mouse_delta
            .x
            .mul_add(-SENSITIVITY, transform.rotation.y);
        transform.rotation.x = input
            .mouse_delta
            .y
            .mul_add(-SENSITIVITY, transform.rotation.x);

        input.mouse_delta = Vec2::ZERO;

        // -/+ 0.001 so when u look all the way down or all the way up it doesn't invert forward vector
        transform.rotation.x = transform
            .rotation
            .x
            .clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);

        camera.buffer.update(transform.transformation_uniform());
    }
}
