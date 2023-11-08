use wgpu::util::DeviceExt;

use super::Instance;

pub(crate) struct RawInstanceBuffer<C, I: Instance<C>> {
    _dummy: std::marker::PhantomData<C>, 
    pub(crate) instances: Vec<I::Raw>, 
}
impl<C, I: Instance<C>> RawInstanceBuffer<C, I> {
    pub fn new() -> Self { Self {
        _dummy: std::marker::PhantomData, 
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

pub(crate) struct InstanceBuffer<C, I: Instance<C>> {
    _dummy: std::marker::PhantomData<C>, 
    buffer: Vec<Option<I>>, 
}
impl<C, I: Instance<C>> InstanceBuffer<C, I> {
    pub fn new() -> Self { Self { 
        _dummy: std::marker::PhantomData,     
        buffer: Vec::new() 
    } }
    pub fn push(&mut self, instance: I) {
        self.buffer.push(Some(instance))
    }
    pub(crate) fn finish(
        &mut self, 
        ria: &mut RawInstanceBuffer<C, I>, 
        context: &mut C, 
        value: &I::T, 
    ) {
        ria.instances.clear();
        self.buffer.iter_mut()
            .filter_map(|v| v.take())
            .for_each(|i| ria.instances.push(i.as_raw(
                context, 
                value, 
            )));
        self.buffer.clear();
    }
}

pub struct InstanceArray<C, I: Instance<C>> {
    raw: RawInstanceBuffer<C, I>, 
    bake: InstanceBuffer<C, I>, 
}
impl<C, I: Instance<C>> InstanceArray<C, I> {
    pub fn new() -> Self { Self {
        raw: RawInstanceBuffer::new(),
        bake: InstanceBuffer::new(),
    }}

    pub fn push(&mut self, instance: I) {
        self.bake.push(instance)
    }

    pub fn finish(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        context: &mut C, 
        value: &I::T, 
    ) -> wgpu::Buffer {
        self.bake.finish(
            &mut self.raw, 
            context, 
            value, 
        );
        self.raw.gen_buffer(gfx)
    }

    pub fn len(&self) -> usize { self.raw.instances.len() }
}