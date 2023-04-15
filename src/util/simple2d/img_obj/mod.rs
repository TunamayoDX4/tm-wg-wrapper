use wgpu::{
    Buffer, 
    BindGroupLayout, 
    VertexAttribute, 
    VertexBufferLayout, 
    VertexStepMode, 
    BufferAddress, 
    vertex_attr_array, 
    util::{DeviceExt, BufferInitDescriptor}, 
    RenderPipeline, 
    RenderPipelineDescriptor, 
    BindGroup, 
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
}
impl Instance for ImgObjInstance {
    type Raw = ImgObjInstanceRaw;
    type Arv = types::Texture;

    fn as_raw(&self, v: &Self::Arv) -> Self::Raw {
        let position = self.position;
        let size = self.size;
        let rotation = [
            self.rotation.cos(), 
            self.rotation.sin(), 
        ];
        let tex_coord = std::array::from_fn(|i|
            self.tex_coord[i] / v.texture_size[i]
        );
        let tex_size = std::array::from_fn(|i|
            self.tex_size[i] / v.texture_size[i]
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
impl InstanceGen<ImgObjInstance> for ImgObjInstance {
    fn generate(&self) -> ImgObjInstance { *self }
}

/// 画像用レンダラで共有される値
pub struct ImgObjRenderShared {
    pipeline: RenderPipeline, 
    vertex_buffer: Buffer, 
    index_buffer: Buffer, 
    pub diffuse_bg_layout: BindGroupLayout, 
}
impl ImgObjRenderShared {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        camera_bindgroup_layout: &BindGroupLayout, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // シェーダー
        let shader = gfx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("image shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("ferris_shooting_imgobj.wgsl").into(), 
                ),
            }
        );
        
        // バーテックスバッファ
        let vertex_buffer = gfx.device.create_buffer_init(
            &BufferInitDescriptor { 
                label: Some("vertex buffer"), 
                contents: bytemuck::cast_slice(raw_param::VERTICES), 
                usage: wgpu::BufferUsages::VERTEX 
            }
        );

        // インデックスバッファ
        let index_buffer = gfx.device.create_buffer_init(
            &BufferInitDescriptor { 
                label: Some("index buffer"), 
                contents: bytemuck::cast_slice(raw_param::INDICES), 
                usage: wgpu::BufferUsages::INDEX,  
            }
        );

        // テクスチャ用のバインドグループレイアウト
        let diffuse_bg_layout = gfx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("diffuse bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Texture { 
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2, 
                            multisampled: false,  
                        }, 
                        count: None, 
                    }, 
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, 
                        visibility: wgpu::ShaderStages::FRAGMENT, 
                        ty: wgpu::BindingType::Sampler(
                            wgpu::SamplerBindingType::Filtering, 
                        ), 
                        count: None, 
                    }, 
                ],
            }
        );

        // パイプラインのレイアウト
        let pipeline_layout = gfx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"), 
                bind_group_layouts: &[
                    &diffuse_bg_layout, 
                    &camera_bindgroup_layout, 
                ], 
                push_constant_ranges: &[]
            }
        );

        // パイプライン
        let pipeline = gfx.device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("sample pipeline"), 
                layout: Some(&pipeline_layout), 
                vertex: wgpu::VertexState {
                    module: &shader, 
                    entry_point: "vs_main", 
                    buffers: &[
                        raw_param::Vertex::desc(), 
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

        Ok(Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            diffuse_bg_layout, 
        })
    }

    pub fn rendering(
        &self, 
        gfx: &crate::ctx::gfx::GfxCtx, 
        view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder, 
        camera_bg: &BindGroup, 
        renderer: &mut ImgObjRender, 
    ) {
        renderer.rendering(
            gfx, 
            view, 
            encoder, 
            &self.pipeline, 
            camera_bg, 
            &self.vertex_buffer, 
            &self.index_buffer
        )
    }
}

/// 画像用レンダラ
pub struct ImgObjRender {
    texture: types::Texture, 
    instances: RawInstanceArray<ImgObjInstance>, 
    instance_buffer: Buffer, 
}
impl ImgObjRender {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        bind_group_layout: &BindGroupLayout, 
        texture: impl AsRef<std::path::Path>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // テクスチャのロード
        let texture = types::Texture::new(
            gfx, 
            bind_group_layout, 
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
    pub fn update_instances<'a, T: InstanceGen<ImgObjInstance>>(
        &mut self, 
        instances: impl Iterator<Item = &'a T>, 
    ) {
        self.instances.modify(instances, &self.texture)
    }

    /// 描画
    pub fn rendering(
        &mut self, 
        gfx: &crate::ctx::gfx::GfxCtx,
        view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder, 
        pipeline: &wgpu::RenderPipeline, 
        camera_bg: &wgpu::BindGroup, 
        vb: &wgpu::Buffer, 
        ib: &wgpu::Buffer, 
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

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &self.texture.bind_group, &[]);
        render_pass.set_bind_group(1, camera_bg, &[]);
        render_pass.set_vertex_buffer(0, vb.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(
            0..raw_param::INDICES.len() as _, 
            0, 
            0..self.instances.len() as _
        );
    }
}