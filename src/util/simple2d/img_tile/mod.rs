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
use std::mem::size_of;
use super::{
    types::{
        self, 
        Instance, 
        InstanceGen, 
    }, 
    raw_param::{
        self, 
        InstanceRaw, 
        RawInstanceArray, 
    }, 
};

/// タイル画像の生インスタンス
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImgTileInstanceRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub tex_coord: [f32; 2], 
    pub tex_size: [f32; 2], 
}
impl ImgTileInstanceRaw {
    const ATTRIBS: [VertexAttribute; 4] = vertex_attr_array![
        5 => Float32x2, 
        6 => Float32x2, 
        7 => Float32x2, 
        8 => Float32x2, 
    ];
}
impl InstanceRaw for ImgTileInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress, 
            step_mode: VertexStepMode::Instance, 
            attributes: &Self::ATTRIBS, 
        }
    }
}

/// タイル画像のインスタンス
#[derive(Debug, Clone, Copy)]
pub struct ImgTileInstance {
    pub position: [i64; 2], 
    pub tex_coord: [f32; 2], 
    pub tex_size: [f32; 2], 
    pub tex_rev: [bool; 2], 
}
impl Instance for ImgTileInstance {
    type Raw = ImgTileInstanceRaw;
    type Arv = super::types::Texture;

    fn as_raw(&self, v: &Self::Arv) -> Self::Raw {
        let position = std::array::from_fn(|i| self.position[i] as f32);
        let tex_coord = std::array::from_fn(|i|
            self.tex_coord[i] / v.texture_size[i] + if self.tex_rev[i] {
                self.tex_size[i] / v.texture_size[i]
            } else {
                0.
            }
        );
        let tex_size = std::array::from_fn(|i|
            self.tex_size[i] / v.texture_size[i] * if self.tex_rev[i] { -1. } else { 1. }
        );

        ImgTileInstanceRaw { 
            position, 
            size: v.texture_size, 
            tex_coord, 
            tex_size 
        }
    }
}
impl InstanceGen<ImgTileInstance> for ImgTileInstance {
    fn generate(&self) -> ImgTileInstance { *self }
}

/// 画像を使ったタイルの描画構造体で共有される値
pub struct ImgTileRenderShared {
    pipeline: RenderPipeline, 
}
impl ImgTileRenderShared {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        camera: &super::S2DCamera, 
        image_shared: &super::ImagedShared, 
    ) -> Self {
        // シェーダモジュールの読み込み
        let shader = gfx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("image shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("imaged_tile.wgsl").into(), 
                ), 
            }
        );

        // パイプラインレイアウトの初期化
        let pipeline_layout = gfx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"), 
                bind_group_layouts: &[
                    &image_shared.diffuse, 
                    &camera.bg_layout, 
                ], 
                push_constant_ranges: &[]
            }
        );

        // パイプラインの初期化
        let pipeline = gfx.device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("sample pipeline"), 
                layout: Some(&pipeline_layout), 
                vertex: wgpu::VertexState {
                    module: &shader, 
                    entry_point: "vs_main", 
                    buffers: &[
                        raw_param::Vertex::desc(), 
                        ImgTileInstanceRaw::desc(), 
                    ], 
                }, 
                fragment: Some(wgpu::FragmentState {
                    module: &shader, 
                    entry_point: "fs_main", 
                    targets: &[Some(wgpu::ColorTargetState { 
                        format: gfx.config.format, 
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING), 
                        write_mask: wgpu::ColorWrites::all() 
                    })]
                }), 
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, 
                    strip_index_format: None, 
                    front_face: wgpu::FrontFace::Ccw, 
                    cull_mode: Some(wgpu::Face::Back), 
                    unclipped_depth: false, 
                    polygon_mode: wgpu::PolygonMode::Fill, 
                    conservative: false, 
                }, 
                depth_stencil: None, 
                multisample: wgpu::MultisampleState {
                    count: 1, 
                    mask: !0, 
                    alpha_to_coverage_enabled: false, 
                }, 
                multiview: None, 
            }
        );

        Self {
            pipeline
        }
    }
}

/// 画像用レンダラ
pub struct ImgTileRender {
    texture: types::Texture, 
    instances: RawInstanceArray<ImgTileInstance>, 
    instance_buffer: Buffer, 
}
impl ImgTileRender {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        imaged_shared: &super::ImagedShared, 
        texture: impl AsRef<std::path::Path>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // テクスチャのロード
        let texture = types::Texture::new(
            gfx, 
            &imaged_shared.diffuse, 
            texture, 
        )?;

        // インスタンスの生成
        let mut instances = RawInstanceArray::new();

        // インスタンスバッファの初期化
        let instance_buffer = instances.gen_buffer(gfx).unwrap();

        Ok(Self {
            texture, 
            instances, 
            instance_buffer, 
        })
    }

    /// インスタンスの更新
    pub fn update_instances<'a, T: InstanceGen<ImgTileInstance>>(
        &mut self, 
        instances: impl Iterator<Item = &'a T>, 
    ) {
        self.instances.modify(instances, &self.texture)
    }
}
impl super::Simple2DRender for ImgTileRender {
    type Shared<'a> = (
        &'a super::SquareShared, 
        &'a super::ImagedShared, 
        &'a ImgTileRenderShared, 
    );

    fn rendering<'a>(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &super::S2DCamera, 
        shared: Self::Shared<'a>, 
    ) {
        if let Some(buffer) = self.instances.gen_buffer(gfx) {
            self.instance_buffer = buffer;
        }

        let mut render_pass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("render pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                    view, 
                    resolve_target: None, 
                    ops: wgpu::Operations { 
                        load: wgpu::LoadOp::Load, 
                        store: true 
                    } 
                })], 
                depth_stencil_attachment: None, 
            }
        );

        render_pass.set_pipeline(&shared.2.pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
        render_pass.set_bind_group(1, &camera.bg, &[]);
        render_pass.set_vertex_buffer(
            0, shared.0.vertex.slice(..)
        );
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            shared.0.index.slice(..), wgpu::IndexFormat::Uint16
        );
        render_pass.draw_indexed(
            0..raw_param::INDICES.len() as _, 
            0, 
            0..self.instances.len() as _
        );
    }   
}