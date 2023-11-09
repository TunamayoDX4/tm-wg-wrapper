use super::*;

/// アトラスを用いた描画構造体で共有される値
pub struct AtlasObjRenderShared {
    pub(super) pipeline: wgpu::RenderPipeline, 
}
impl AtlasObjRenderShared {
    pub fn new(
        gfx: &crate::ctx::gfx::GfxCtx, 
        camera: &super::super::super::S2DCamera, 
        image: &super::super::super::ImagedShared, 
    ) -> Self {
        let shader = gfx.device.create_shader_module(
            wgpu::ShaderModuleDescriptor { 
                label: Some("atlas 2d shader"), 
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("atlas_object.wgsl").into()
                ) 
            }
        );

        // パイプラインレイアウトの初期化
        let pipeline_layout = gfx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("pipeline layout"), 
                bind_group_layouts: &[
                    &image.diffuse, 
                    &camera.bg_layout, 
                ], 
                push_constant_ranges: &[]
            }
        );

        // パイプラインの初期化
        let pipeline = gfx.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("sample pipeline"), 
                layout: Some(&pipeline_layout), 
                vertex: wgpu::VertexState {
                    module: &shader, 
                    entry_point: "vs_main", 
                    buffers: &[
                        super::super::super::raw::TexedVertex::desc(), 
                        super::instance::AtlasObjInstanceRaw::desc(), 
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