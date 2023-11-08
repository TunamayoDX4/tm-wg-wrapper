pub mod buffer;

pub trait InstanceRaw where
    Self: Send + Sync + Sized + Copy + bytemuck::Pod + bytemuck::Zeroable
{
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

pub trait Instance<C>: Send + Sync + Sized {
    type Raw: InstanceRaw;
    type T;

    fn as_raw(
        self, 
        context: &mut C, 
        value: &Self::T, 
    ) -> Self::Raw;
}

pub trait InstanceGen<C, I: Instance<C>>: Send + Sync {
    fn generate(&self, instances: &mut buffer::InstanceArray<C, I>);
}

pub trait InstanceModifier<I: Instance<Self>>: Send + Sync + Sized + 'static {
    fn modify(
        &mut self, 
        instance: &mut I, 
    );
}
impl<I: Instance<Self>> InstanceModifier<I> for () {
    /// なにもしない
    fn modify(
        &mut self, 
        _instance: &mut I, 
    ) {}
}