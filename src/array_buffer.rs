use crate::render_context::RenderContext;
use bytemuck::{Pod, Zeroable, cast_slice};
use std::any::type_name;
use std::marker::PhantomData;
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, ShaderStages,
};

#[derive(Debug)]
pub struct TypedArrayBuffer<T: Pod + Zeroable> {
    inner: Buffer,
    buffer_type: BufferBindingType,
    len: usize,
    _t: PhantomData<T>,
    context: RenderContext,
}

impl<T: Pod + Zeroable> TypedArrayBuffer<T> {
    pub fn new_uniform(context: RenderContext, data: &[T]) -> Self {
        Self {
            inner: Self::create_buffer(
                &context,
                data,
                BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            ),
            buffer_type: BufferBindingType::Uniform,
            _t: PhantomData,
            len: data.len().next_power_of_two(),
            context,
        }
    }

    pub fn new_storage(context: RenderContext, data: &[T]) -> Self {
        Self {
            inner: Self::create_buffer(
                &context,
                data,
                BufferUsages::STORAGE | BufferUsages::COPY_DST,
            ),
            buffer_type: BufferBindingType::Storage { read_only: true },
            _t: PhantomData,
            len: data.len().next_power_of_two(),
            context,
        }
    }

    #[must_use]
    pub const fn as_layout_entry(
        &self,
        binding: u32,
        visibility: ShaderStages,
    ) -> BindGroupLayoutEntry {
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

    #[must_use]
    pub fn as_bind_group_entry(&self, binding: u32) -> BindGroupEntry<'_> {
        BindGroupEntry {
            binding,
            resource: self.inner.as_entire_binding(),
        }
    }

    pub fn update(&mut self, data: &[T]) -> bool {
        if data.len() > self.len {
            self.inner = Self::create_buffer(&self.context, data, self.inner.usage());
            self.len = data.len().next_power_of_two();
            return true;
        }
        self.context
            .queue
            .write_buffer(&self.inner, 0, cast_slice(data));
        false
    }

    fn create_buffer(context: &RenderContext, data: &[T], usage: BufferUsages) -> Buffer {
        let buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some(type_name::<T>()),
            size: (data.len().next_power_of_two() * size_of::<T>()) as u64,
            usage,
            mapped_at_creation: false,
        });
        context.queue.write_buffer(&buffer, 0, cast_slice(data));
        buffer
    }
}
