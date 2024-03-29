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
    instance::{
        Instance, 
        InstanceGen, 
        InstanceRaw, 
        buffer::InstanceArray, 
    }, 
    shared::{
        S2DCamera, 
        ImagedShared, 
        SquareShared, 
    }, 
    types::Texture, 
    raw::{
        TexedVertex, 
        INDICES, 
    }, 
};

/// GPUで処理される生のインスタンス
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImgObjInstanceRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: [f32; 2], 
    pub tex_coord: [f32; 2], 
    pub tex_size: [f32; 2], 
}
impl ImgObjInstanceRaw {
    const ATTRIBS: [VertexAttribute; 5] = vertex_attr_array![
        5 => Float32x2, 
        6 => Float32x2, 
        7 => Float32x2, 
        8 => Float32x2, 
        9 => Float32x2, 
    ];
}
impl InstanceRaw for ImgObjInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress, 
            step_mode: VertexStepMode::Instance, 
            attributes: &Self::ATTRIBS, 
        }
    }
}

/// インスタンス
#[derive(Debug, Clone, Copy)]
pub struct ImgObjInstance {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: f32, 
    pub tex_coord: [f32; 2], 
    pub tex_size: [f32; 2], 
    pub tex_rev: [bool; 2], 
}
impl Instance<Texture> for ImgObjInstance {
    type Raw = ImgObjInstanceRaw;

    fn as_raw(self, value: &Texture) -> Self::Raw {
        let position = self.position;
        let size = self.size;
        let rotation = [
            self.rotation.cos(), 
            self.rotation.sin(), 
        ];
        let tex_coord = std::array::from_fn(|i|
            self.tex_coord[i] / value.texture_size[i] + if self.tex_rev[i] {
                self.tex_size[i] / value.texture_size[i]
            } else {
                0.
            }
        );
        let tex_size = std::array::from_fn(|i|
            self.tex_size[i] / value.texture_size[i] * if self.tex_rev[i] { -1. } else { 1. }
        );

        ImgObjInstanceRaw { 
            position, 
            size, 
            rotation, 
            tex_coord, 
            tex_size 
        }
    }
    
}
impl InstanceGen<Texture, ImgObjInstance> for ImgObjInstance {
    fn generate(
        &self, 
        instances: &mut super::instance::buffer::InstanceArray<
            Texture, 
            ImgObjInstance
        >, 
    ) {
        instances.push(*self)
    }
}

/// 画像を使ったオブジェクトの描画構造体で共有される値
pub struct ImgObjRenderShared {
    pipeline: RenderPipeline, 
}
impl ImgObjRenderShared {
    pub fn new(
        gfx: &crate::ctx::gfx::WGPUCtx, 
        camera: &S2DCamera, 
        image_shared: &ImagedShared, 
    ) -> Self {
        // シェーダモジュールの読み込み
        let shader = gfx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("image shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("imaged_object.wgsl").into(), 
                )
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
                        TexedVertex::desc(), 
                        ImgObjInstanceRaw::desc(), 
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
            pipeline, 
        }
    }
}

/// 画像用レンダラ
pub struct ImgObjRender {
    texture: Texture, 
    instances: InstanceArray<Texture, ImgObjInstance>, 
    instance_buffer: Buffer, 
}
impl ImgObjRender {
    pub fn new<C: std::ops::Deref<Target = [u8]>>(
        gfx: &crate::ctx::gfx::WGPUCtx, 
        imaged_shared: &ImagedShared, 
        image: image::ImageBuffer<image::Rgba<u8>, C>, 
    ) -> Self {
        // テクスチャのロード
        let texture = Texture::from_image(
            gfx, 
            &imaged_shared.diffuse, 
            image, 
        );

        // インスタンスの生成
        let mut instances = InstanceArray::new();

        // インスタンスバッファの初期化
        let instance_buffer = instances.finish(gfx, &texture);

        Self {
            texture, 
            instances, 
            instance_buffer, 
        }
    }

    pub fn from_image(
        gfx: &crate::ctx::gfx::WGPUCtx, 
        imaged_shared: &ImagedShared, 
        texture: impl AsRef<std::path::Path>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // テクスチャのロード
        let texture = Texture::new(
            gfx, 
            &imaged_shared.diffuse, 
            texture, 
        )?;

        // インスタンスの生成
        let mut instances = InstanceArray::new();

        // インスタンスバッファの初期化
        let instance_buffer = instances.finish(gfx, &texture);

        Ok(Self {
            texture, 
            instances, 
            instance_buffer, 
        })
    }

    /// テクスチャの更新用の可変参照の取得
    pub fn get_texture(&mut self) -> &mut Texture { &mut self.texture }

    /// インスタンスの更新
    pub fn push_instance<'a, T: InstanceGen<Texture, ImgObjInstance> + 'a>(
        &mut self, 
        instance: &T, 
    ) {
        instance.generate(&mut self.instances);
    }
}
impl<GCd: Send + Sync> super::Simple2DRender<GCd> for ImgObjRender {
    type Shared<'a> = (
        &'a SquareShared, 
        &'a ImagedShared, 
        &'a ImgObjRenderShared, 
    );

    fn rendering<'a>(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx<GCd>, 
        encoder: &mut wgpu::CommandEncoder, 
        view: &wgpu::TextureView, 
        camera: &S2DCamera, 
        shared: Self::Shared<'a>, 
    ) {
        self.instance_buffer = self.instances.finish(
            &gfx.wgpu_ctx, 
            &self.texture
        );

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
            0..INDICES.len() as _, 
            0, 
            0..self.instances.len() as _
        );
    }
    
}