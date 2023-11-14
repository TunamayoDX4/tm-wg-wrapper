use wgpu::util::DeviceExt;

use super::Instance;

pub(crate) struct RawInstanceBuffer<V, I: Instance<V>> {
    pub(crate) instances: Vec<I::Raw>, 
}
impl<V, I: Instance<V>> RawInstanceBuffer<V, I> {
    pub fn new() -> Self { Self {
        instances: Vec::new(),
    }}

    pub fn gen_buffer(
        &mut self, 
        gfx: &crate::ctx::gfx::WGPUCtx, 
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

pub(crate) struct InstanceBuffer<V, I: Instance<V>> {
    _dummy: std::marker::PhantomData<V>, 
    buffer: Vec<Option<I>>, 
}
impl<V, I: Instance<V>> InstanceBuffer<V, I> {
    pub fn new() -> Self { Self { 
        _dummy: std::marker::PhantomData, 
        buffer: Vec::new(), 
    } }
    pub fn push(&mut self, instance: I) {
        self.buffer.push(Some(instance))
    }
    pub(crate) fn finish(
        &mut self, 
        ria: &mut RawInstanceBuffer<V, I>, 
        value: &V, 
    ) {
        ria.instances.clear();
        self.buffer.iter_mut()
            .filter_map(|v| v.take())
            .for_each(|i| ria.instances.push(i.as_raw(value)));
        self.buffer.clear();
    }
}

pub struct InstanceArray<V, I: Instance<V>> {
    raw: RawInstanceBuffer<V, I>, 
    bake: InstanceBuffer<V, I>, 
}
impl<V, I: Instance<V>> InstanceArray<V, I> {
    pub fn new() -> Self { Self {
        raw: RawInstanceBuffer::new(),
        bake: InstanceBuffer::new(),
    }}

    pub fn push(&mut self, instance: I) {
        self.bake.push(instance)
    }

    pub fn finish(
        &mut self, 
        gfx: &crate::ctx::gfx::WGPUCtx, 
        value: &V, 
    ) -> wgpu::Buffer {
        self.bake.finish(&mut self.raw, value);
        self.raw.gen_buffer(gfx)
    }

    pub fn len(&self) -> usize { self.raw.instances.len() }
}