pub mod buffer;

pub trait InstanceRaw where
    Self: Send + Sync + Sized + Copy + bytemuck::Pod + bytemuck::Zeroable
{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub trait Instance<V>: Send + Sync + Sized {
    type Raw: InstanceRaw;

    fn as_raw(self, value: &V) -> Self::Raw;
}

pub trait InstanceGen<V, I: Instance<V>>: Send + Sync {
    fn generate(&self, instances: &mut buffer::InstanceArray<V, I>);
}