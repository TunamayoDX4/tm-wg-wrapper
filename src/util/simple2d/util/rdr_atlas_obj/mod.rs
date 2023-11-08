use std::hash::Hash;

use wgpu::{
    Buffer, 
    VertexAttribute, 
    VertexBufferLayout, 
    VertexStepMode, 
    BufferAddress, 
    vertex_attr_array, 
    RenderPipeline, 
    RenderPipelineDescriptor, 
};
use super::atlas::{
    Atlas, 
    AtlasController, 
};
use super::super::types::Texture;
use super::super::instance::{
    Instance, 
    InstanceRaw, 
    buffer::InstanceArray, 
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtlasObjInstanceRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: [f32; 2], 
    pub tex_coord: [f32; 2], 
    pub tex_size: [f32; 2], 
    pub atlas_object_coord: [f32; 2], 
    pub atlas_object_size: [f32; 2], 
}
impl InstanceRaw for AtlasObjInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        todo!()
    }
}

pub struct AtlasObjInstance {
    pub atlas_id: usize, 
    pub position: nalgebra::Point2<f32>, 
    pub size: nalgebra::Vector2<f32>, 
    pub rotation: f32, 
    pub tex_rev: [bool; 2], 
    pub tex_coord: nalgebra::Point2<f32>, 
    pub tex_size: nalgebra::Vector2<f32>, 
    pub atlas_object_coord: nalgebra::Point2<f32>, 
    pub atlas_object_size: nalgebra::Vector2<f32>, 
}
impl<
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, u8, K, [[f32; 2]; 2], 
    >, 
> Instance<AtlasRenderingModule<K, I>> for AtlasObjInstance {
    type Raw = AtlasObjInstanceRaw;
    fn as_raw(self, value: &AtlasRenderingModule<K, I>) -> Self::Raw {
        let (
            _, 
            uv, 
            _, 
            _, 
        ) = value.atlas.get(self.atlas_id).unwrap();

        todo!()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtlasUniform {
    pub atlas_size: [f32; 2], 
}

pub struct AtlasRenderingModule<K, I> where
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, u8, K, [[f32; 2]; 2], 
    >, 
{
    atlas: Atlas<4, u8, K, [[f32; 2]; 2], I>, 
    texture: Texture, 
}