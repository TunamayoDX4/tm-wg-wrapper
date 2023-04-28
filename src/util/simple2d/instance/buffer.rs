use wgpu::util::DeviceExt;

use super::{
    Instance, 
};

pub(crate) struct RawInstanceBuffer<I: Instance> {
    pub(crate) instances: Vec<I::Raw>, 
}
impl<I: Instance> RawInstanceBuffer<I> {
    pub fn new() -> Self { Self {
        instances: Vec::new(),
    }}

    pub fn gen_buffer(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
    ) -> wgpu::Buffer {
        gfx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(self.instances.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }
}

pub(crate) struct InstanceBuffer<I: Instance> {
    buffer: Vec<Option<I>>, 
}
impl<I: Instance> InstanceBuffer<I> {
    pub fn new() -> Self { Self { buffer: Vec::new() } }
    pub fn push(&mut self, instance: I) {
        self.buffer.push(Some(instance))
    }
    pub(crate) fn finish(
        &mut self, 
        ria: &mut RawInstanceBuffer<I>, 
        value: &I::T, 
    ) {
        ria.instances.clear();
        self.buffer.iter_mut()
            .filter_map(|v| v.take())
            .for_each(|i| ria.instances.push(i.as_raw(value)));
        self.buffer.clear();
    }
}

pub struct InstanceArray<I: Instance> {
    raw: RawInstanceBuffer<I>, 
    bake: InstanceBuffer<I>, 
}
impl<I: Instance> InstanceArray<I> {
    pub(crate) fn new() -> Self { Self {
        raw: RawInstanceBuffer::new(),
        bake: InstanceBuffer::new(),
    }}

    pub fn push(&mut self, instance: I) {
        self.bake.push(instance)
    }

    pub(crate) fn finish(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        value: &I::T, 
    ) -> wgpu::Buffer {
        self.bake.finish(&mut self.raw, value);
        self.raw.gen_buffer(gfx)
    }

    pub(crate) fn len(&self) -> usize { self.raw.instances.len() }
}