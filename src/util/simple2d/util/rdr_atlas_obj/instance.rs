use std::{
    hash::Hash, 
    mem::size_of, 
};
use wgpu::{
    VertexAttribute, 
    VertexStepMode, 
    VertexBufferLayout, 
    BufferAddress, 
    vertex_attr_array, 
};
use super::super::super::instance::{
    Instance, 
    InstanceRaw, 
};
use super::super::atlas::AtlasController;

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
impl AtlasObjInstanceRaw {
    const ATTRIB: [VertexAttribute; 7] = vertex_attr_array![
        5 => Float32x2, 
        6 => Float32x2, 
        7 => Float32x2, 
        8 => Float32x2, 
        9 => Float32x2, 
        10 => Float32x2, 
        11 => Float32x2, 
    ];
}
impl InstanceRaw for AtlasObjInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout { 
            array_stride: size_of::<Self>() as BufferAddress, 
            step_mode: VertexStepMode::Instance, 
            attributes: &Self::ATTRIB
        }
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
}
impl<
    K: Eq + Hash + Send + Sync + Sized + 'static, 
    I: AtlasController<
        4, u8, K, (nalgebra::Point2<f32>, nalgebra::Vector2<f32>), 
    > + Send + Sync, 
> Instance<super::AtlasRenderingModule<K, I>> for AtlasObjInstance {
    type Raw = AtlasObjInstanceRaw;
    fn as_raw(self, value: &super::AtlasRenderingModule<K, I>) -> Self::Raw {
        let (
            _, 
            (
                coord, 
                size
            ), 
            _, 
            _, 
        ) = value.atlas.get(self.atlas_id).unwrap();
        Self::Raw {
            position: self.position.into(),
            size: self.size.into(),
            rotation: [
                self.rotation.cos(), 
                self.rotation.sin()
            ],
            tex_coord: std::array::from_fn(
                |i| self.tex_coord[i] / size[i] + if self.tex_rev[i] {
                    self.tex_size[i] / size[i]
                } else {
                    0.
                }
            ),
            tex_size: std::array::from_fn(
                |i| self.tex_size[i] / size[i] 
                    * if self.tex_rev[i] { -1. } else { 1. }
            ),
            atlas_object_coord: (*coord).into(),
            atlas_object_size: (*size).into(),
        }
    }
}