/// 生のカメラのデータ
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: [f32; 2], 
    pub _dummy: [f32; 2], 
}

/// テクスチャ付き頂点のデータ
#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexedVertex {
    pub position: [f32; 4], 
    pub tex_coord: [f32; 2], 
}
impl TexedVertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x4, 
        1 => Float32x2, 
    ];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// 四角形のテクスチャ付き頂点配列
pub const TEXED_VERTICES: &[TexedVertex] = &[
    TexedVertex {
        position: [-1., 1., 0., 1.], 
        tex_coord: [0., 0.], 
    }, 
    TexedVertex {
        position: [-1., -1., 0., 1.], 
        tex_coord: [0., 1.], 
    }, 
    TexedVertex {
        position: [1., -1., 0., 1.], 
        tex_coord: [1., 1.], 
    }, 
    TexedVertex {
        position: [1., 1., 0., 1.], 
        tex_coord: [1., 0.], 
    }, 
];

/// 四角形の描画のためのインデックスバッファ
pub const INDICES: &[u16] = &[
    0, 1, 3, 
    1, 2, 3, 
];