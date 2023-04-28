pub mod buffer;

pub trait InstanceRaw where
    Self: Send + Sync + Sized + Copy + bytemuck::Pod + bytemuck::Zeroable
{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub trait Instance: Send + Sync + Sized {
    type Raw: InstanceRaw;
    type T;

    fn as_raw(self, value: &Self::T) -> Self::Raw;
}

pub trait InstanceGen<I: Instance>: Send + Sync {
    fn generate(&self, instances: &mut buffer::InstanceArray<I>);
}