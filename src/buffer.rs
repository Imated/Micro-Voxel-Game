use crate::render_context::RenderContext;
use bytemuck::{Pod, Zeroable, cast_slice};
use std::any::type_name;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages,
    ShaderStages,
};

#[derive(Debug)]
pub struct TypedBuffer<T: Pod + Zeroable> {
    inner: Buffer,
    buffer_type: BufferBindingType,
    _t: PhantomData<T>,
}

impl<T: Pod + Zeroable> TypedBuffer<T> {
    pub fn new_uniform(context: &RenderContext, data: T) -> Self {
        Self {
            inner: Self::create_buffer(
                context,
                data,
                BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ),
            buffer_type: BufferBindingType::Uniform,
            _t: PhantomData,
        }
    }

    pub fn new_storage(context: &RenderContext, data: T) -> Self {
        Self {
            inner: Self::create_buffer(
                context,
                data,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ),
            buffer_type: BufferBindingType::Storage { read_only: false },
            _t: PhantomData,
        }
    }

    pub fn as_layout_entry(&self, binding: u32, visibility: ShaderStages) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility,
            ty: BindingType::Buffer {
                ty: self.buffer_type,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn as_bind_group_entry(&self, binding: u32) -> BindGroupEntry<'_> {
        BindGroupEntry {
            binding,
            resource: self.inner.as_entire_binding(),
        }
    }

    pub fn update(&self, context: &RenderContext, data: T) {
        context
            .queue
            .write_buffer(&self.inner, 0, cast_slice(&[data]));
    }

    fn create_buffer(context: &RenderContext, data: T, usage: BufferUsages) -> Buffer {
        context.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(type_name::<T>()),
            usage,
            contents: cast_slice(&[data]),
        })
    }
}
