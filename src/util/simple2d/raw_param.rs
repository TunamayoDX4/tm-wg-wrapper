use std::mem::size_of;
use wgpu::{
    VertexBufferLayout, 
    BufferAddress, 
    VertexStepMode, 
    VertexAttribute, 
    vertex_attr_array, util::DeviceExt, 
};

/* ------ 型の宣言 ------ */

/// 生のインスタンス
pub trait InstanceRaw: Send + Sync + Sized + Copy + bytemuck::Pod + bytemuck::Zeroable {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}

/// 生のインスタンス配列
pub struct RawInstanceArray<I: super::types::Instance> {
    modified: bool, 
    instances: Vec<I::Raw>, 
}
impl<I: super::types::Instance> RawInstanceArray<I> {
    pub fn new() -> Self { Self {
        modified: true, 
        instances: Vec::new(), 
    } }

    pub fn modify<'a, T: super::types::InstanceGen<I>>(
        &mut self, 
        instances: impl Iterator<Item = &'a T>, 
        v: &I::Arv, 
    ) {
        self.modified = true;
        self.instances.clear();
        instances
            .map(|i| i.generate().as_raw(v))
            .for_each(|i| self.instances.push(i));
    }

    pub fn gen_buffer(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
    ) -> Option<wgpu::Buffer> {
        if self.modified {
            self.modified = false;
            Some(gfx.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("instance buffer"), 
                    contents: bytemuck::cast_slice(self.instances.as_slice()), 
                    usage: wgpu::BufferUsages::VERTEX, 
                }
            ))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize { self.instances.len() }
}

/// 頂点データ
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 4], 
    pub tex_coord: [f32; 2], 
}
impl Vertex { 
    pub const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![
        0 => Float32x4, 
        1 => Float32x2, 
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress, 
            step_mode: VertexStepMode::Vertex, 
            attributes: &Self::ATTRIBS, 
        }
    }
}

/// GPUで処理される生のカメラ
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: [f32; 2], 
    pub _dummy: [f32; 2], 
}

/* ------ 定数の宣言 ------ */

/// 四角形の頂点 
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1., 0., 1.], 
        tex_coord: [0., 0.], 
    }, 
    Vertex {
        position: [-1., -1., 0., 1.], 
        tex_coord: [0., 1.], 
    }, 
    Vertex {
        position: [1., -1., 0., 1.], 
        tex_coord: [1., 1.], 
    }, 
    Vertex {
        position: [1., 1., 0., 1.], 
        tex_coord: [1., 0.], 
    }, 
];

/// 四角形の描画のためのインデックス
pub const INDICES: &[u16] = &[
    0, 1, 3, 
    1, 2, 3, 
];