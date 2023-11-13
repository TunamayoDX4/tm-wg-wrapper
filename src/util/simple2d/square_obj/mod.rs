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
        SquareShared, 
    }, 
    raw::{
        TexedVertex, 
        INDICES, 
    }, 
};

/// GPUで処理される生のインスタンス
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SqObjInstanceRaw {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: [f32; 2], 
    pub color: [f32; 4], 
}
impl SqObjInstanceRaw {
    const ATTRIBS: [VertexAttribute; 4] = vertex_attr_array![
        5 => Float32x2, 
        6 => Float32x2, 
        7 => Float32x2, 
        8 => Float32x4, 
    ];
}
impl InstanceRaw for SqObjInstanceRaw {
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
pub struct SqObjInstance {
    pub position: [f32; 2], 
    pub size: [f32; 2], 
    pub rotation: f32, 
    pub color: [f32; 4], 
}
impl Instance<()> for SqObjInstance {
    type Raw = SqObjInstanceRaw;

    fn as_raw(self, _value: &()) -> Self::Raw {
        let position = self.position;
        let size = self.size;
        let rotation = [
            self.rotation.cos(), 
            self.rotation.sin(), 
        ];

        SqObjInstanceRaw { 
            position, 
            size, 
            rotation, 
            color: self.color, 
        }
    }
    
}
impl InstanceGen<(), SqObjInstance> for SqObjInstance {
    fn generate(
        &self, 
        instances: &mut super::instance::buffer::InstanceArray<
            (), 
            SqObjInstance
        >, 
    ) {
        instances.push(*self)
    }
}

/// 画像を使ったオブジェクトの描画構造体で共有される値
pub struct SqObjRenderShared {
    pipeline: RenderPipeline, 
}
impl SqObjRenderShared {
    pub fn new<GCd: Send + Sync>(
        gfx: &crate::ctx::gfx::WGPUCtx, 
        camera: &S2DCamera, 
        polygon_mode: wgpu::PolygonMode, 
    ) -> Self {
        // シェーダモジュールの読み込み
        let shader = gfx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("image shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("square_object.wgsl").into(), 
                )
            }
        );

        // パイプラインレイアウトの初期化
        let pipeline_layout = gfx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"), 
                bind_group_layouts: &[
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
                        SqObjInstanceRaw::desc(), 
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
                    polygon_mode, 
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
pub struct SqObjRender {
    instances: InstanceArray<(), SqObjInstance>, 
    instance_buffer: Buffer, 
}
impl SqObjRender {
    pub fn new<C: std::ops::Deref<Target = [u8]>, GCd: Send + Sync>(
        gfx: &crate::ctx::gfx::GfxCtx<GCd>, 
    ) -> Self {

        // インスタンスの生成
        let mut instances = InstanceArray::new();

        // インスタンスバッファの初期化
        let instance_buffer = instances.finish(gfx, &());

        Self {
            instances, 
            instance_buffer, 
        }
    }

    pub fn from_image<GCd: Send + Sync>(
        gfx: &crate::ctx::gfx::GfxCtx<GCd>, 
    ) -> Result<Self, Box<dyn std::error::Error>> {

        // インスタンスの生成
        let mut instances = InstanceArray::new();

        // インスタンスバッファの初期化
        let instance_buffer = instances.finish(gfx, &());

        Ok(Self {
            instances, 
            instance_buffer, 
        })
    }

    /// インスタンスの更新
    pub fn push_instance<'a, T: InstanceGen<(), SqObjInstance> + 'a>(
        &mut self, 
        instance: &T, 
    ) {
        instance.generate(&mut self.instances);
    }
}
impl<GCd: Send + Sync> super::Simple2DRender<GCd> for SqObjRender {
    type Shared<'a> = (
        &'a SquareShared, 
        &'a SqObjRenderShared, 
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
            gfx, 
            &()
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

        render_pass.set_pipeline(&shared.1.pipeline);
        render_pass.set_bind_group(0, &camera.bg, &[]);
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